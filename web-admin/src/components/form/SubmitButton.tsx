import Spinner from "src/components/Spinner";
import React from "react";

const SubmitButton = ({ isSubmitting, label, extraClass, btnClass }) => {
  return (
    <span className={`block w-full rounded-md shadow-sm ${extraClass}`}>
      <button
        disabled={isSubmitting}
        type="submit"
        className={`btn-orange w-full justify-center ${btnClass}`}
      >
        {!isSubmitting && <span className="h-6">{label}</span>}
        {isSubmitting && <Spinner className="h-6 w-6" />}
      </button>
    </span>
  );
};

SubmitButton.defaultProps = {
  extraClass: "",
  btnClass: "",
};

export default SubmitButton;
