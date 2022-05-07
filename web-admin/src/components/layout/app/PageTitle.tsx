import { Link } from "react-router-dom";
import React from "react";

const PageTitle = ({ title, breadcrumbs, buttons }) => {
  const backButton =
    breadcrumbs.length > 1 ? breadcrumbs[breadcrumbs.length - 2] : null;

  return (
    <div>
      <div>
        {backButton && (
          <nav className="sm:hidden">
            <Link to={backButton.href}>
              <button className="flex items-center text-sm leading-5 font-medium text-gray-500 hover:text-gray-700 transition duration-150 ease-in-out">
                <svg
                  className="flex-shrink-0 -ml-1 mr-1 h-5 w-5 text-gray-400"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path
                    fillRule="evenodd"
                    d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z"
                    clipRule="evenodd"
                  />
                </svg>
                Back
              </button>
            </Link>
          </nav>
        )}

        {breadcrumbs.length > 0 && (
          <nav className="hidden sm:flex items-center text-sm leading-5 font-medium">
            {breadcrumbs.map((breadcrumb, index) => {
              const includeSeparator = index < breadcrumbs.length - 1;

              return (
                <React.Fragment key={index}>
                  <Link to={breadcrumb.href}>
                    <button className="text-gray-500 hover:text-gray-700 transition duration-150 ease-in-out">
                      {breadcrumb.text}
                    </button>
                  </Link>
                  {includeSeparator && (
                    <svg
                      className="flex-shrink-0 mx-2 h-5 w-5 text-gray-400"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fillRule="evenodd"
                        d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                        clipRule="evenodd"
                      />
                    </svg>
                  )}
                </React.Fragment>
              );
            })}
          </nav>
        )}
      </div>
      <div className="mt-2 md:flex md:items-center md:justify-between">
        <div className="flex-1 min-w-0">
          <h2 className="text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:leading-9 sm:truncate">
            {title}
          </h2>
        </div>
        {buttons.length > 0 && (
          <div className="mt-4 flex-shrink-0 flex md:mt-0 md:ml-4">
            {buttons.map((button, index) => {
              return (
                <span key={index} className="shadow-sm rounded-md">
                  {button.href && (
                    <Link to={button.href}>
                      <button
                        type="button"
                        className={`inline-flex items-center px-4 py-2 border border-gray-300 text-sm leading-5 font-medium rounded-md text-gray-700 bg-white hover:text-gray-500 focus:outline-none focus:ring-blue focus:border-blue-300 active:text-gray-800 active:bg-gray-50 transition duration-150 ease-in-out ${button.class}`}
                      >
                        {button.text}
                      </button>
                    </Link>
                  )}

                  {button.onClick && (
                    <button
                      type="button"
                      onClick={button.onClick}
                      className={`inline-flex items-center px-4 py-2 border border-gray-300 text-sm leading-5 font-medium rounded-md text-gray-700 bg-white hover:text-gray-500 focus:outline-none focus:ring-blue focus:border-blue-300 active:text-gray-800 active:bg-gray-50 transition duration-150 ease-in-out ${button.class}`}
                    >
                      {button.text}
                    </button>
                  )}
                </span>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

PageTitle.defaultProps = {
  breadcrumbs: [],
  buttons: [],
};

export default PageTitle;
