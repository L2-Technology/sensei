import Spinner from "src/components/Spinner";
import React from "react";

const FooterSubmitButton = ({ isSubmitting, label, extraClass, btnClass }) => {
  return (
    <div className="px-4 py-3 bg-gray-50 text-right sm:px-6">
      <span className={`inline-flex rounded-md shadow-sm ${extraClass}`}>
        <button
          disabled={isSubmitting}
          type="submit"
          className={`inline-flex justify-center py-2 px-4 border border-transparent text-sm leading-5 font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-500 focus:outline-none focus:border-indigo-700 focus:ring-indigo active:bg-indigo-700 transition duration-150 ease-in-out ${btnClass}`}
        >
          {!isSubmitting && label}
          {isSubmitting && <Spinner className="h-6 w-6" />}
        </button>
      </span>
    </div>
  );
};

FooterSubmitButton.defaultProps = {
  extraClass: "",
  btnClass: "",
};

export default FooterSubmitButton;
