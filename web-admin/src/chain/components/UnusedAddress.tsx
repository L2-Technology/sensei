import getUnusedAddress from "../queries/getUnusedAddress";
import { useQuery } from "react-query";
import QRCode from "react-qr-code";
import copy from "copy-to-clipboard";
import { useNotification } from "src/contexts/notification";
import { CheckIcon, ClipboardCopyIcon } from "@heroicons/react/outline";

const CopiedAddressNotification = () => {
  return (
    <div className="">
      <p className="text-sm font-medium text-gray-50">Address Copied to Clipboard</p>
    </div>
  );
};

const UnusedAddress = () => {
  const { showNotification, hideNotification } = useNotification()
  
  const { isLoading, isError, data } = useQuery(
    "unused_address",
    getUnusedAddress
  );

  if (isLoading || isError) {
    return null;
  }

  let { address } = data;

  return (
    <div>
      <div
        style={{ width: "145px" }}
        className="bg-white shadow overflow-hidden sm:rounded-lg p-2 cursor-pointer hover:scale-110 transition-all ease-in-out duration-200"
        onClick={() => {
          copy(address)
          showNotification({
            component: <CopiedAddressNotification/>,
            iconComponent: <CheckIcon className="h-6 w-6 text-orange" />
          })
          setTimeout(hideNotification, 1000)
        }}
      >
        <QRCode
          value={address}
          size={128}
          bgColor={"#FFFFFF"}
          fgColor={"#000000"}
          level={"L"}
        />
      </div>
    </div>
  );
};

export default UnusedAddress;
