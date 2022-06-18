import payInvoice from "../mutations/payInvoice";
import decodeInvoice from "../mutations/decodeInvoice";
import { Form, TextArea } from "../../components/form";
import { z } from "zod";
import { useQueryClient } from "react-query";
import { useState } from "react";
import { truncateMiddle } from "src/utils/capitalize";

export const CreateInvoiceInput = z.object({
  invoice: z.string(),
});

const DecodedInvoice = ({ decodedInvoice }) => {
  return (
    <div className="mt-4 mb-4 p-4 bg-plum rounded">
      Pay <span className="font-bold text-gray-200">{decodedInvoice.amount / 1000} sats</span> 
      &nbsp;for <span className="font-bold text-gray-200">{decodedInvoice.description}</span>
      &nbsp;to <span  className="font-bold text-gray-200">{truncateMiddle(decodedInvoice.payeePubKey, 15)}</span>
    </div>
  )
}

const PayInvoiceForm = () => {
  const [decodedInvoice, setDecodedInvoice] = useState(null)
  const [rawInvoice, setRawInvoice] = useState("")

  const queryClient = useQueryClient();

  const onInvoiceChange = async (event) => {
    setRawInvoice(event.target.value)
    try {
      const invoice = await decodeInvoice(event.target.value)
      setDecodedInvoice(invoice)
    } catch (e) {}
  }

  return (
    <Form
      submitText="Pay Invoice"
      schema={CreateInvoiceInput}
      noticePosition="top"
      layout="default"
      resetAfterSuccess={true}
      onSubmit={async () => {
        try {
          await payInvoice(rawInvoice);
          setDecodedInvoice(null)
          queryClient.invalidateQueries("payments")
          
        } catch (e) {
          // TODO: handle error
        }
      }}
    >
      <TextArea label="Invoice" name="invoice" onChange={onInvoiceChange} extraClass="h-20"/>
      {decodedInvoice && <DecodedInvoice decodedInvoice={decodedInvoice} />}
    </Form>
  );
};

export default PayInvoiceForm;
