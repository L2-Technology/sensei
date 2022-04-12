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
        <label className="block text-sm leading-5 mb-1 font-medium text-light-plum">
          {label}
        </label>
        <select
          {...input}
          disabled={submitting}
          {...props}
          ref={ref}
          className="input"
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
