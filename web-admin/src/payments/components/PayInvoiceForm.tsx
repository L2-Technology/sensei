import { useNavigate } from "react-router";
import payInvoice from "../mutations/payInvoice";
import { Form, TextArea } from "../../components/form";
import { z } from "zod";
import { useQueryClient } from "react-query";

export const CreateInvoiceInput = z.object({
  invoice: z.string(),
});

const PayInvoiceForm = () => {
  const queryClient = useQueryClient();
  let navigate = useNavigate();

  return (
    <Form
      submitText="Pay Invoice"
      schema={CreateInvoiceInput}
      noticePosition="top"
      layout="default"
      resetAfterSuccess={true}
      onSubmit={async ({ invoice }) => {
        try {
          await payInvoice(invoice);
          queryClient.invalidateQueries("payments")
          
        } catch (e) {
          // TODO: handle error
        }
      }}
    >
      <TextArea label="Invoice" name="invoice" />
    </Form>
  );
};

export default PayInvoiceForm;
