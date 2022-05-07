import sensei from "../../utils/sensei";

const deleteAccessToken = async (id: number) => {
  return await sensei.deleteAccessToken(id);
};

export default deleteAccessToken;
