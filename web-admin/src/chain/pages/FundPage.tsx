import UnusedAddress from "../components/UnusedAddress";

const FundPage = () => {
  return (
    <div className="py-6">
      <div className="">
        <h1 className="text-2xl font-semibold text-plum-light text-center">
          Fund Node
        </h1>
      </div>
      <div className="">
        <div className="py-4">
          <UnusedAddress />
        </div>
      </div>
    </div>
  );
};

export default FundPage;
