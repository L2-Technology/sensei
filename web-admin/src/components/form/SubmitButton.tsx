import Spinner from "src/components/Spinner";
import React from "react";

const SubmitButton = ({ isSubmitting, label, extraClass, btnClass }) => {
  return (
    <span className={`block w-full rounded-md shadow-sm ${extraClass}`}>
      <button
        disabled={isSubmitting}
        type="submit"
        className={`w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-orange hover:bg-orange-hover focus:outline-none focus:border-blue-200 focus:ring-indigo active:bg-orange transition duration-150 ease-in-out ${btnClass}`}
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
