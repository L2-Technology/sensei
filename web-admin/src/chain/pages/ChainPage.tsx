import OnChainBalance from "../components/OnChainBalance";
import TransactionsList from "src/transactions/components/TransactionsList";
import UnusedAddress from "../components/UnusedAddress";
const ChainPage = () => {
  return (
    <div className="py-6">
      <div className="">
        <h1 className="text-2xl font-semibold text-plum-light">Fund Node</h1>
      </div>
      <div className="">
        <div className="py-4">
          <div className="mb-8">
            <UnusedAddress />
          </div>
        </div>
      </div>
      <div className="">
        <h1 className="text-2xl font-semibold text-plum-light">Balance</h1>
      </div>
      <div className="">
        <div className="py-4">
          <div className="mb-8">
            <OnChainBalance />
          </div>
        </div>
      </div>

      <div className="">
        <h1 className="text-2xl font-semibold text-plum-light">Transactions</h1>
      </div>
      <div className="">
        <div className="py-4">
          <div className="mb-8 bg-gray-accent2 -mx-4 sm:mx-0 sm:rounded-xl  overflow-x-auto">
            <TransactionsList />
          </div>
        </div>
      </div>
    </div>
  );
};

export default ChainPage;
