import React, { PropsWithoutRef, ReactNode } from "react";
import { useField } from "react-final-form";

export interface InputProps
  extends PropsWithoutRef<JSX.IntrinsicElements["input"]> {
  name: string;
  label?: string;
  extraClass?: string;
  className?: string;
  prepend?: string;
  append?: string;
  type?: "text" | "password" | "email" | "number" | "hidden";
  badge?: ReactNode;
  innerImage?: ReactNode;
  arrayElement?: boolean;
  parse?: any;
  validate?: (value) => any;
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

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  (
    {
      name,
      label,
      extraClass,
      badge,
      outerProps,
      className,
      prepend,
      append,
      innerImage,
      arrayElement,
      parse,
      validate,
      ...props
    },
    ref
  ) => {
    let config = parse || validate ? {} : undefined;
    config = parse ? { ...config, parse } : config;
    config = validate ? { ...config, validate } : config;

    const {
      input,
      meta: { touched, error, submitting },
    } = useField(name, config);

    // const hasError = (dirty || submitFailed) && (error || submitError)
    const hasError = touched && error;

    return (
      <div
        {...outerProps}
        className={`${arrayElement ? "" : "mb-5"} ${className}`}
      >
        {label && (
          <div className="flex justify-between">
            <label className="block text-sm font-medium leading-5 text-light-plum">
              {label}
            </label>
            {badge && badge}
          </div>
        )}

        <div
          className={`${label ? "mt-1" : ""} ${
            innerImage || append || prepend ? "relative" : ""
          } rounded-xl shadow-sm flex`}
        >
          {prepend && (
            <span className="inline-flex items-center px-3 rounded-l-md border border-r-0 border-gray-300 bg-gray-50 text-gray-500 sm:text-sm">
              {prepend}
            </span>
          )}
          {innerImage && (
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              {innerImage}
            </div>
          )}
          <input
            {...input}
            disabled={submitting}
            {...props}
            ref={ref}
            autoComplete="off"
            className={`input ${
              hasError
                ? "border-red-300 focus:border-red-500 focus:ring-red-500"
                : ""
            } ${prepend ? "rounded-l-none" : ""} ${append ? "pr-12" : ""} ${
              innerImage ? "pl-10" : ""
            } ${extraClass}`}
          />
          {append && (
            <div className="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
              <span className="text-gray-500 sm:text-sm">{append}</span>
            </div>
          )}
        </div>
        {hasError && <ErrorMessages errors={error} />}
      </div>
    );
  }
);

Input.defaultProps = {
  type: "text",
  extraClass: "",
  className: "",
  placeholder: "",
  disabled: false,
  autoFocus: false,
};

export default Input;
