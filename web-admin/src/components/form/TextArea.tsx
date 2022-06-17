import React, { PropsWithoutRef } from "react";
import { useField } from "react-final-form";

export interface TextAreaProps
  extends PropsWithoutRef<JSX.IntrinsicElements["textarea"]> {
  name: string;
  label?: string;
  extraClass?: string;
  className?: string;
  outerProps?: PropsWithoutRef<JSX.IntrinsicElements["div"]>;
}

const ErrorMessages = ({ errors }) => {
  return (
    <>
      {errors.map((error, index) => {
        return (
          <div key={index} role="alert" className="mt-2 text-sm text-red-600">
            {error}
          </div>
        );
      })}
    </>
  );
};

export const TextArea = React.forwardRef<HTMLTextAreaElement, TextAreaProps>(
  ({ name, label, extraClass, outerProps, className, ...props }, ref) => {
    const {
      input,
      meta: { dirty, error, submitFailed, submitError, submitting },
    } = useField(name);

    let myOnChange = props.onChange ? (e) => {props.onChange(e); input.onChange(e) } : input.onChange;
    
    const hasError = (dirty || submitFailed) && (error || submitError);

    return (
      <div {...outerProps} className={`mb-5 ${className}`}>
        {label && (
          <div className="flex justify-between">
            <label className="block text-sm font-medium leading-5 text-light-plum">
              {label}
            </label>
          </div>
        )}
        <div className="mt-1 rounded-xl shadow-sm ">
          <textarea
            {...input}
            disabled={submitting}
            {...props}
            onChange={myOnChange}
            ref={ref}
            className={`form-input block border rounded-xl w-full bg-plum text-light-plum transition duration-150 ease-in-out sm:text-sm sm:leading-5 flex-1 ${
              hasError
                ? "border-red-500 focus:border-red-500 focus:ring-red-500"
                : "border-gray-plum-100 focus:border-orange  focus:ring-orange"
            } ${extraClass}`}
          ></textarea>
        </div>
        {hasError && <ErrorMessages errors={error} />}
      </div>
    );
  }
);

TextArea.defaultProps = {
  extraClass: "",
  className: "",
  placeholder: "",
  disabled: false,
  autoFocus: false,
};

export default TextArea;
