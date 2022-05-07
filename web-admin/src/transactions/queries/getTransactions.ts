import sensei from "../../utils/sensei";

const getTransactions = async ({ page, searchTerm, take }) => {
  return await sensei.getTransactions({ page, searchTerm, take });
};

export default getTransactions;
