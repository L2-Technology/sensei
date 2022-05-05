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
    <div className="">
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
          className="btn-ghost text-sm"
        >
          Copy
        </button>
        <button
          type="button"
          onClick={hideNotification}
          className="btn-ghost text-sm"
        >
          Dismiss
        </button>
      </div>
    </div>
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
      <Input autoFocus label="Description" name="description" />
      <Input min={1} label="Amount Millisats" name="amountMillisats" type="number" />
    </Form>
  );
};

export default CreateInvoiceForm;
