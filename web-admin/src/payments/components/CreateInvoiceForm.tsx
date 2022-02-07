import { useNavigate } from "react-router";
import createInvoice from "../mutations/createInvoice";
import { Form, Input } from "../../components/form";
import { useQueryClient } from "react-query";
import * as z from "zod";
import { useNotification } from "../../contexts/notification";
import copy from "copy-to-clipboard";

export const CreateInvoiceInput = z.object({
  amountMillisats: z.string(),
  description: z.string(),
});

const NewInvoiceNotification = ({ invoice }) => {
  const { hideNotification } = useNotification();

  return (
    <>
      <p className="text-sm font-medium text-gray-50">Invoice Created</p>
      <p className="mt-1 text-sm text-light-plum">
        Easily copy invoice to clipboard or dismiss and find it later below
      </p>
      <div className="mt-3 flex space-x-7">
        <button
          type="button"
          onClick={() => {
            copy(invoice);
            hideNotification();
          }}
          className="bg-plum-50 p-1 text-light-plum rounded-md text-sm font-medium  focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
        >
          Copy
        </button>
        <button
          type="button"
          onClick={hideNotification}
          className="bg-plum-50 p-1 text-light-plum  rounded-md text-sm font-medium  hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
        >
          Dismiss
        </button>
      </div>
    </>
  );
};

const CreateInvoiceForm = () => {
  const queryClient = useQueryClient();
  const { showNotification } = useNotification();

  return (
    <Form
      submitText="Create Invoice"
      schema={CreateInvoiceInput}
      noticePosition="top"
      layout="default"
      resetAfterSuccess={true}
      initialValues={{ description: "", amountMillisats: "" }}
      onSubmit={async ({ description, amountMillisats }) => {
        try {
          const { invoice } = await createInvoice(
            parseInt(amountMillisats, 10),
            description
          );

          queryClient.invalidateQueries("payments");

          showNotification({
            component: <NewInvoiceNotification invoice={invoice} />,
          });
        } catch (e) {
          // TODO: handle error
        }
      }}
    >
      <Input label="Description" name="description" />
      <Input label="Amount Millisats" name="amountMillisats" type="number" />
    </Form>
  );
};

export default CreateInvoiceForm;
