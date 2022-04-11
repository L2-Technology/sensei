import { Transition } from "@headlessui/react";
import ClickableDiv from "src/components/layout/app/ClickableDiv";
import { useConfirm } from "src/contexts/confirm";
import React from "react";

const ConfirmModal = () => {
  const {
    hideConfirm,
    isOpen,
    title,
    description,
    ctaText,
    Icon,
    callback,
    callbackData,
    cancelCallback,
    color,
  } = useConfirm();

  const btnClass =
    color === "red"
      ? "bg-red-600 hover:bg-red-500 focus:border-red-700 focus:ring-red"
      : "bg-green-600 hover:bg-green-500 focus:border-green-700 focus:ring-green";

  const iconWrapperClass = color === "red" ? "bg-red-100" : "bg-green-100";

  return (
    <Transition show={isOpen}>
      <div className="fixed bottom-0 inset-x-0 px-4 z-10 pb-4 sm:inset-0 sm:flex sm:items-center sm:justify-center">
        <Transition.Child
          enter="ease-out duration-300"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <ClickableDiv
            clickHandler={() => {
              cancelCallback();
              hideConfirm();
            }}
            className="fixed inset-0 transition-opacity"
          >
            <div className="absolute inset-0 bg-black opacity-50"></div>
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
            className="bg-plum text-plum-light rounded-xl px-4 pt-5 pb-4 overflow-hidden shadow-xl transform transition-all sm:max-w-lg sm:w-full sm:p-6"
            role="dialog"
          >
            <div className="sm:flex sm:items-start">
              <div
                className={`mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full sm:mx-0 sm:h-10 sm:w-10 ${iconWrapperClass}`}
              >
                <Icon />
              </div>
              <div className="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left">
                <h3
                  className="text-lg leading-6 font-medium text-gray-100"
                  id="modal-headline"
                >
                  {title}
                </h3>
                <div className="mt-2">
                  <p className="text-sm leading-5 text-gray-500">
                    {description}
                  </p>
                </div>
              </div>
            </div>
            <div className="mt-5 sm:mt-4 sm:flex sm:flex-row-reverse">
              <span className="flex w-full rounded-md shadow-sm sm:ml-3 sm:w-auto">
                <button
                  type="button"
                  onClick={() => {
                    callback(callbackData);
                    hideConfirm();
                  }}
                  className={`btn-ghost ${btnClass}`}
                >
                  {ctaText}
                </button>
              </span>
              <span className="mt-3 flex w-full rounded-md shadow-sm sm:mt-0 sm:w-auto">
                <button
                  type="button"
                  onClick={() => {
                    cancelCallback();
                    hideConfirm();
                  }}
                  className="btn-ghost"
                >
                  Cancel
                </button>
              </span>
            </div>
          </div>
        </Transition.Child>
      </div>
    </Transition>
  );
};

export default ConfirmModal;
