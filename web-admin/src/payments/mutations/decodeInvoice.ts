import sensei from "../../utils/sensei";

const decodeInvoice = async (invoice: string) => {
  return await sensei.decodeInvoice(invoice);
};

export default decodeInvoice;
