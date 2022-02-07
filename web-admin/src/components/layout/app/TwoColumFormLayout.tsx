import { ReactNode } from "react";

interface TwoColumnForm {
  title: string;
  description: string;
  form: ReactNode;
}

interface TwoColumnFormLayoutProps {
  forms: TwoColumnForm[];
}

const TwoColumnFormLayoutDivider = () => {
  return (
    <div className="hidden sm:block">
      <div className="py-5">
        <div className="border-t border-gray-200"></div>
      </div>
    </div>
  );
};

const TwoColumnFormLayoutItem = ({ title, description, form }) => {
  return (
    <div className="mt-4">
      <div className="md:grid md:grid-cols-3 md:gap-6">
        <div className="md:col-span-1">
          <div className="px-4 sm:px-0 py-4">
            <h3 className="text-lg font-medium leading-6 text-gray-900">
              {title}
            </h3>
            <p className="mt-1 text-sm leading-5 text-gray-600">
              {description}
            </p>
          </div>
        </div>
        <div className="mt-5 md:mt-0 md:col-span-2">{form}</div>
      </div>
    </div>
  );
};

const TwoColumnFormLayout = ({ forms }: TwoColumnFormLayoutProps) => {
  return (
    <>
      {forms.map((form, index) => {
        const isLastForm = index === forms.length - 1;
        return (
          <div key={index}>
            <TwoColumnFormLayoutItem {...form} />
            {!isLastForm && <TwoColumnFormLayoutDivider />}
          </div>
        );
      })}
    </>
  );
};

export default TwoColumnFormLayout;
