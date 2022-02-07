import { Form, FORM_ERROR, Input } from "../../components/form";
import * as z from "zod";
import getConfig from "../queries/getConfig";
import updateConfig from "../mutations/updateConfig";
import { useQuery } from "react-query";
import { useNotification } from "../../contexts/notification";

export const UpdateConfigInput = z.object({
  electrumUrl: z.string(),
});

const ConfigUpdatedNotification = () => {
  const { hideNotification } = useNotification();

  return (
    <>
      <p className="text-sm font-medium text-gray-900">Config Updated</p>
      <p className="mt-1 text-sm text-gray-500">
        Your configuration file has been updated. You will need to start or
        restart nodes for the changes to take affect.
      </p>
      <div className="mt-3 flex space-x-7">
        <button
          type="button"
          onClick={hideNotification}
          className="bg-white rounded-md text-sm font-medium text-gray-700 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
        >
          Got it
        </button>
      </div>
    </>
  );
};

const ConfigForm = () => {
  const { data, isLoading, isError } = useQuery("settings", getConfig);
  const { showNotification } = useNotification();

  if (isLoading) {
    return <div>Loading</div>;
  }
  if (isError) {
    return <div>Error</div>;
  }

  return (

        <div className="relative rounded-md shadow-sm">
          <Form
            initialValues={{ electrumUrl: data.electrumUrl }}
            submitText={"Update Config"}
            schema={UpdateConfigInput}
            noticePosition="top"
            onSubmit={async ({ electrumUrl }) => {
              try {
                await updateConfig({ electrumUrl });
                showNotification({
                  component: <ConfigUpdatedNotification />,
                });
              } catch (error) {
                return { [FORM_ERROR]: error.toString() };
              }
            }}
          >
            <div className="">
              <Input
                className=""
                name="electrumUrl"
                label="Electrum URL"
                autoComplete="off"
              />
            </div>
          </Form>
        </div>
   
  );
};

export default ConfigForm;
