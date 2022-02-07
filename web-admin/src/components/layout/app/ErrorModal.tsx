import { Transition } from "@headlessui/react";
import ClickableDiv from "src/components/layout/app/ClickableDiv";
import { useError } from "src/contexts/error";
import React from "react";

const ErrorModal = () => {
  const { error, hideError } = useError();
  const isOpen = !!error;

  return (
    <Transition show={isOpen}>
      <div className="fixed bottom-0 inset-x-0 px-4 pb-6 sm:inset-0 sm:p-0 sm:flex sm:items-center sm:justify-center">
        <Transition.Child
          enter="ease-out duration-300"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <ClickableDiv
            clickHandler={hideError}
            className="fixed inset-0 transition-opacity"
          >
            <div className="absolute inset-0 bg-gray-500 opacity-75"></div>
          </ClickableDiv>
        </Transition.Child>
        <Transition.Child
          enter="ease-out duration-300"
          enterFrom="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
          enterTo="opacity-100 translate-y-0 sm:scale-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100 translate-y-0 sm:scale-100"
          leaveTo="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
        >
          <div
            className="bg-plum text-light-plum rounded-lg px-4 pt-5 pb-4 overflow-hidden shadow-xl transform transition-all sm:max-w-sm sm:w-full sm:p-6"
            role="dialog"
            aria-modal="true"
            aria-labelledby="modal-headline"
          >
            <div>
              <div className="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-red-100">
                <svg
                  className="h-6 w-6 text-red-500"
                  stroke="currentColor"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth="2"
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
              </div>
              <div className="mt-3 text-center sm:mt-5">
                <h3
                  className="text-lg leading-6 font-medium text-plum-light"
                  id="modal-headline"
                >
                  Something went wrong
                </h3>
                <div className="mt-2">
                  <p className="text-sm leading-5 text-gray-500">{error}</p>
                </div>
              </div>
            </div>
            <div className="mt-5 sm:mt-6">
              <span className="flex w-full rounded-md shadow-sm">
                <button
                  onClick={hideError}
                  type="button"
                  className="inline-flex justify-center w-full rounded-md border border-transparent px-4 py-2 bg-gray-600 text-base leading-6 font-medium text-white shadow-sm hover:bg-gray-500 focus:outline-none focus:border-gray-700 focus:ring-gray transition ease-in-out duration-150 sm:text-sm sm:leading-5"
                >
                  OK
                </button>
              </span>
            </div>
          </div>
        </Transition.Child>
      </div>
    </Transition>
  );
};

ErrorModal.defaultProps = {
  title: "Something went wrong",
  ctaText: "Ok",
};

export default ErrorModal;
