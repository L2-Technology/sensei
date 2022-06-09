import getBalance from "../queries/getBalance";
import { useQuery } from "react-query";

const OnChainBalance = () => {
  const { isLoading, isError, data } = useQuery("chain_balance", getBalance);

  if (isLoading || isError) {
    return null;
  }

  let { onchainBalanceSats } = data;

  return (
    <div className="bg-gray-accent2 rounded-xl p-8 max-w-xl mx-auto text-center inline-block">
      <div className="text-5xl font-extrabold">
        {Intl.NumberFormat().format(onchainBalanceSats)}{" "}
        <span className="font-bold">sats</span>
      </div>
    </div>
  );
};

export default OnChainBalance;
