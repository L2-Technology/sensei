import createInvoice from "../mutations/createInvoice";
import { Form, Input } from "../../components/form";
import { useQueryClient } from "react-query";
import { z } from "zod";
import { useNotification } from "../../contexts/notification";
import copy from "copy-to-clipboard";
import { InboxIcon } from "@heroicons/react/outline";

export const CreateInvoiceInput = z.object({
  amountSats: z.string(),
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
      initialValues={{ description: "", amountSats: "" }}
      onSubmit={async ({ description, amountSats }) => {
        try {
          const { invoice } = await createInvoice(
            parseInt(amountSats, 10) * 1000,
            description
          );

          queryClient.invalidateQueries("payments");

          showNotification({
            component: <NewInvoiceNotification invoice={invoice} />,
            iconComponent: <InboxIcon className="h-6 w-6 text-light-plum" aria-hidden="true"/>
          });
        } catch (e) {
          // TODO: handle error
        }
      }}
    >
      <Input autoFocus label="Description" name="description" />
      <Input
        min={1}
        label="Amount Sats"
        name="amountSats"
        type="number"
      />
    </Form>
  );
};

export default CreateInvoiceForm;
