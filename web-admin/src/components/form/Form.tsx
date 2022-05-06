import {
  Fragment,
  Dispatch,
  PropsWithoutRef,
  ReactNode,
  SetStateAction,
  useState,
} from "react";
import {
  Form as FinalForm,
  FormProps as FinalFormProps,
} from "react-final-form";
import { z } from "zod";
import ErrorAlert from "../ErrorAlert";
import FooterSubmitButton from "./FooterSubmitButton";
import SubmitButton from "./SubmitButton";
import SuccessAlert from "../SuccessAlert";
import arrayMutators from "final-form-arrays";
export { FORM_ERROR } from "final-form";

export interface FormProps<S extends z.ZodType<any, any>>
  extends Omit<PropsWithoutRef<JSX.IntrinsicElements["form"]>, "onSubmit"> {
  children?: ReactNode;
  submitText?: string;
  schema?: S;
  decorators?: FinalFormProps<z.infer<S>>["decorators"];
  onSubmit: FinalFormProps<z.infer<S>>["onSubmit"];
  initialValues?: FinalFormProps<z.infer<S>>["initialValues"];
  noticePosition: "top" | "bottom";
  successMessage?: string | ((values: z.infer<S>) => string);
  resetAfterSuccess?: boolean;
  hasFieldArray?: boolean;
  hideSubmitBtn?: boolean;
  layout?: "default" | "card-footer";
  validate?: FinalFormProps<z.infer<S>>["validate"];
}

type successMessageStateType = string | undefined;

type FormLayoutProps = {
  children: ReactNode;
  noticePosition: "top" | "bottom";
  successMessage?: string;
  submitError?: string;
  submitButton: ReactNode;
};

export const DefaultFormLayout = ({
  noticePosition,
  submitError,
  children,
  successMessage,
  submitButton,
}: FormLayoutProps) => {
  return (
    <>
      {noticePosition === "bottom" && children}
      {submitError && <ErrorAlert className="mb-5" message={submitError} />}
      {successMessage && (
        <SuccessAlert className="mb-5" message={successMessage} />
      )}
      {noticePosition === "top" && children}
      {submitButton}
    </>
  );
};

export const CardFooterFormLayout = ({
  noticePosition,
  submitError,
  children,
  successMessage,
  submitButton,
}: FormLayoutProps) => {
  return (
    <div className="shadow sm:rounded-md sm:overflow-hidden">
      <div className="px-4 py-5 bg-white sm:p-6">
        {noticePosition === "bottom" && children}
        {submitError && <ErrorAlert className="mb-5" message={submitError} />}
        {successMessage && (
          <SuccessAlert className="mb-5" message={successMessage} />
        )}
        {noticePosition === "top" && children}
      </div>
      {submitButton}
    </div>
  );
};

export function Form<S extends z.ZodType<any, any>>({
  children,
  submitText,
  schema,
  initialValues,
  decorators = [],
  onSubmit,
  hasFieldArray,
  noticePosition,
  successMessage,
  resetAfterSuccess,
  hideSubmitBtn,
  validate,
  layout = "default",
  ...props
}: FormProps<S>) {
  const [message, setMessage]: [
    successMessageStateType,
    Dispatch<SetStateAction<successMessageStateType>>
  ] = useState(undefined);

  const LayoutComponent =
    layout === "card-footer" ? CardFooterFormLayout : DefaultFormLayout;
  const SubmitButtonComponent =
    layout === "card-footer" ? FooterSubmitButton : SubmitButton;
  const mutators = hasFieldArray ? arrayMutators : {};

  const defaultValidationFunction = (values) => {
    if (!schema) return;
    try {
      schema.parse(values);
    } catch (error) {
      return error.formErrors.fieldErrors;
    }
  };

  const validationFunction = validate || defaultValidationFunction;

  return (
    <FinalForm
      initialValues={initialValues}
      validate={validationFunction}
      mutators={{ ...mutators }}
      decorators={decorators}
      onSubmit={onSubmit}
      render={({ handleSubmit, submitting, submitError, form }) => (
        <form
          onSubmit={(event) => {
            setMessage(undefined);
            handleSubmit(event)?.then((errors) => {
              if (!errors) {
                if (successMessage) {
                  const msg =
                    typeof successMessage === "string"
                      ? successMessage
                      : successMessage(form.getState().values);
                  setMessage(msg);
                }
                if (resetAfterSuccess) {
                  if (document.activeElement instanceof HTMLElement) {
                    document.activeElement.blur();
                  }
                  form.reset();
                  Object.keys(initialValues as object).forEach(
                    (initialValueKey) => {
                      form.resetFieldState(initialValueKey);
                    }
                  );
                  form.reset();
                }
              }
            });
          }}
          className="form"
          {...props}
          autoComplete="chrome-off"
        >
          <LayoutComponent
            noticePosition={noticePosition}
            submitError={submitError}
            successMessage={message}
            children={children}
            submitButton={
              hideSubmitBtn ? (
                <Fragment />
              ) : (
                <SubmitButtonComponent
                  isSubmitting={submitting}
                  label={submitText}
                />
              )
            }
          />
        </form>
      )}
    />
  );
}

export default Form;
