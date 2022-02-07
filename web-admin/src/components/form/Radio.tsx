/* eslint-disable jsx-a11y/no-noninteractive-element-interactions */

import React, { ReactNode, PropsWithoutRef } from "react";
import { useField } from "react-final-form";

export interface RadioProps
  extends PropsWithoutRef<JSX.IntrinsicElements["input"]> {
  name: string;
  label: string;
  value: string;
  initialValue: String | null;
  labelChecked: string | ReactNode | null;
  extraClass?: string;
  outerProps?: PropsWithoutRef<JSX.IntrinsicElements["div"]>;
}

export const Radio = React.forwardRef<HTMLInputElement, RadioProps>(
  (
    {
      name,
      label,
      value,
      initialValue,
      labelChecked,
      extraClass,
      outerProps,
      ...props
    },
    ref
  ) => {
    const {
      input,
      meta: { submitting },
    } = useField(name, { type: "radio", value });

    const isInitialValue = initialValue && value === initialValue;

    return (
      <div {...outerProps} className="mb-5">
        <label onClick={input.onChange} onKeyPress={input.onChange}>
          <input {...input} {...props} disabled={submitting} ref={ref} />
          <span className="pl-3">
            {label}
            {isInitialValue && labelChecked && labelChecked}
          </span>
        </label>
      </div>
    );
  }
);

Radio.defaultProps = {
  extraClass: "",
  outerProps: {},
};

export default Radio;
