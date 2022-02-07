import { Link } from "react-router-dom";
import React from "react";

const LinkButton = ({ btnHref, btnText, btnClass }) => {
  return (
    <Link to={btnHref}>
      <Button btnText={btnText} btnClass={btnClass} btnOnClick={() => {}} />
    </Link>
  );
};

const Button = ({ btnOnClick, btnText, btnClass }) => {
  return (
    <span className="inline-flex rounded-md shadow-sm">
      <button
        type="button"
        onClick={btnOnClick}
        className={`relative inline-flex items-center px-4 py-2 border border-transparent text-sm leading-5 font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-500 focus:outline-none focus:ring-indigo focus:border-indigo-700 active:bg-indigo-700 ${btnClass}`}
      >
        {btnText}
      </button>
    </span>
  );
};

const CardHeaderWithAction = ({
  title,
  btnHref,
  btnText,
  btnClass,
  showAction,
  btnOnClick,
}) => {
  return (
    <div className="-ml-4 -mt-2 flex items-center justify-between flex-wrap sm:flex-nowrap">
      <div className="ml-4 mt-2">
        <h3 className="text-lg leading-6 font-medium text-gray-900">{title}</h3>
      </div>
      {showAction && (
        <div className="ml-4 mt-2 flex-shrink-0">
          {btnHref && (
            <LinkButton
              btnHref={btnHref}
              btnText={btnText}
              btnClass={btnClass}
            />
          )}
          {btnOnClick && (
            <Button
              btnOnClick={btnOnClick}
              btnText={btnText}
              btnClass={btnClass}
            />
          )}
        </div>
      )}
    </div>
  );
};

CardHeaderWithAction.defaultProps = {
  btnClass: "",
  showAction: true,
  btnHref: undefined,
  btnOnClick: undefined,
};

export default CardHeaderWithAction;
