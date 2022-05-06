import sensei from "../../utils/sensei";

const getAccesstokens = async ({ page, searchTerm, take }) => {
  return await sensei.getAccessTokens({ page, searchTerm, take });
};

export default getAccesstokens;
