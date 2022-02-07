import React from "react";

const Button = ({ label, extraClass, btnClass, onClick, btnColor }) => {
  const colorClass =
    btnColor === "indigo"
      ? "text-white bg-indigo-600 hover:bg-indigo-500 focus:border-indigo-700 focus:ring-indigo active:bg-indigo-700"
      : "text-black bg-gray-200 hover:bg-gray-100 focus:border-gray-300 focus:ring-gray active:bg-gray-300";

  return (
    <span className={`block w-full rounded-md shadow-sm ${extraClass}`}>
      <button
        onClick={onClick}
        className={`w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md focus:outline-none transition duration-150 ease-in-out ${colorClass} ${btnClass}`}
      >
        <span className="h-6">{label}</span>
      </button>
    </span>
  );
};

Button.defaultProps = {
  extraClass: "",
  btnClass: "",
  btnColor: "indigo",
};

export default Button;
