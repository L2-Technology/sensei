import { Transition } from "@headlessui/react";
import ClickableDiv from "./ClickableDiv";
import { useModal } from "../../../contexts/modal";
import React from "react";

const Modal = () => {
  const { hideModal, component, isOpen } = useModal();

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
            clickHandler={hideModal}
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
            className="bg-plum text-light-plum rounded-xl px-4 pt-5 pb-4 overflow-hidden shadow-xl transform transition-all sm:max-w-lg sm:w-full sm:p-6"
            role="dialog"
          >
            {component}
          </div>
        </Transition.Child>
      </div>
    </Transition>
  );
};

export default Modal;
