import getUnusedAddress from "../queries/getUnusedAddress";
import { useQuery } from "react-query";
import QRCode from "react-qr-code";

const UnusedAddress = () => {
  const { isLoading, isError, data } = useQuery(
    "unused_address",
    getUnusedAddress
  );

  if (isLoading || isError) {
    return null;
  }

  let { address } = data;

  return (
    <div className="">
      <div
        style={{ width: "305px" }}
        className="bg-white shadow overflow-hidden sm:rounded-lg p-6 mx-auto"
      >
        <QRCode
          value={address}
          size={256}
          bgColor={"#FFFFFF"}
          fgColor={"#000000"}
          level={"L"}
        />
      </div>
      <div className="text-center text-lg md:text-xl lg:text-2xl m-8">
        {address}
      </div>
    </div>
  );
};

export default UnusedAddress;
