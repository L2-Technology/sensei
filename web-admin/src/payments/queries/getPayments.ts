import sensei from "../../utils/sensei";

const getPayments = async ({ page, searchTerm, take, origin, status }) => {
  return await sensei.getPayments({
    filter: { origin, status },
    pagination: { page, searchTerm, take },
  });
};

export default getPayments;
