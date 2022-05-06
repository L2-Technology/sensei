import sensei from "../../utils/sensei";

const createInvoice = async (amountMillisats: number, description: string) => {
  return await sensei.createInvoice(amountMillisats, description);
};

export default createInvoice;
