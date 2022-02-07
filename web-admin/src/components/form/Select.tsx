import React, { PropsWithoutRef } from "react";
import { useField } from "react-final-form";

export interface SelectOption {
  value: string;
  text: string;
}

export interface SelectProps
  extends PropsWithoutRef<JSX.IntrinsicElements["select"]> {
  name: string;
  label: string;
  options: SelectOption[];
  extraClass?: string;
  outerProps?: PropsWithoutRef<JSX.IntrinsicElements["div"]>;
}

export const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  (
    { name, label, options, extraClass, outerProps, className, ...props },
    ref
  ) => {
    const {
      input,
      meta: { touched, error, submitError, submitting },
    } = useField(name);

    const hasError = touched && (error || submitError);

    return (
      <div {...outerProps} className={`mb-5 ${className}`}>
        <label className="block text-sm leading-5 font-medium text-light-plum">
          {label}
        </label>
        <select
          {...input}
          disabled={submitting}
          {...props}
          ref={ref}
          className="bg-plum text-light-plum mt-1 form-select block w-full pl-3 pr-10 py-2 text-base leading-6 border-plum-200 focus:outline-none focus:ring-blue focus:border-blue-300 sm:text-sm sm:leading-5"
        >
          {options.map((option: SelectOption) => (
            <option key={option.value} value={option.value}>
              {option.text}
            </option>
          ))}
        </select>
        {hasError && (
          <div role="alert" className="mt-2 text-sm text-red-600">
            {error || submitError}
          </div>
        )}
      </div>
    );
  }
);

Select.defaultProps = {
  extraClass: "",
  outerProps: {},
  className: "",
};

export default Select;
