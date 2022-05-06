import { useNavigate } from "react-router";
import payInvoice from "../mutations/payInvoice";
import { Form, TextArea } from "../../components/form";
import { z } from "zod";

export const CreateInvoiceInput = z.object({
  invoice: z.string(),
});

const PayInvoiceForm = () => {
  let navigate = useNavigate();

  return (
    <Form
      submitText="Pay Invoice"
      schema={CreateInvoiceInput}
      noticePosition="top"
      layout="default"
      onSubmit={async ({ invoice }) => {
        try {
          await payInvoice(invoice);
          navigate("/admin/payments");
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
