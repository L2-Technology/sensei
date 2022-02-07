import BackLink from "src/components/layout/app/BackLink";

interface LinkType {
  text: string;
  href: string;
}

interface ActionType {
  text: string;
  onClick: () => void;
}

interface SimpleSectionHeaderProps {
  title: string;
  description?: string;
  backLink?: LinkType;
  action?: ActionType;
}

const SimpleSectionHeader = ({
  title,
  backLink,
  description,
  action,
}: SimpleSectionHeaderProps) => {
  const actionContainerClass = action
    ? "space-y-3 sm:flex sm:items-center sm:justify-between sm:space-x-4 sm:space-y-0"
    : "";
  return (
    <div className={`pb-5 border-b border-gray-200 ${actionContainerClass}`}>
      <div>
        {backLink && <BackLink text={backLink.text} href={backLink.href} />}
        <h3 className="text-2xl leading-6 font-medium text-gray-900">
          {title}
        </h3>
        {description && (
          <p className="max-w-4xl text-sm leading-5 text-gray-500">
            {description}
          </p>
        )}
      </div>
      {action && (
        <div className="self-end">
          <span className="shadow-sm rounded-md">
            <button
              onClick={action.onClick}
              type="button"
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm leading-5 font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-500 focus:outline-none focus:ring-indigo focus:border-indigo-700 active:bg-indigo-700 transition duration-150 ease-in-out"
            >
              {action.text}
            </button>
          </span>
        </div>
      )}
    </div>
  );
};

export default SimpleSectionHeader;
