import sensei from "../../utils/sensei";

const payInvoice = async (invoice: string) => {
  return await sensei.payInvoice(invoice);
};

export default payInvoice;
