import React from "react";

const DefinitionList = ({ title, description, items }) => {
  return (
    <div className="bg-white shadow overflow-hidden sm:rounded-lg">
      <div className="px-4 py-5 border-b border-gray-200 sm:px-6">
        <h3 className="text-lg leading-6 font-medium text-gray-900">{title}</h3>
        <p className="mt-1 max-w-2xl text-sm leading-5 text-gray-500">
          {description}
        </p>
      </div>
      <div className="px-4 py-5 sm:p-0">
        <dl>
          {items.map((item, index) => {
            return (
              <div
                key={index}
                className={`sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6 sm:py-5 ${
                  index > 0 ? "mt-8 sm:mt-0 sm:border-t sm:border-gray-200" : ""
                }`}
              >
                <dt className="text-sm leading-5 font-medium text-gray-500">
                  {item.attribute}
                </dt>
                <dd className="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-2">
                  {item.value}
                </dd>
              </div>
            );
          })}
        </dl>
      </div>
    </div>
  );
};

export default DefinitionList;
