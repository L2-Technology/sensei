import PaymentsList from "../components/PaymentsList";
const PaymentsPage = () => {
  return (
    <div className="py-6">
      <div className="">
        <h1 className="text-2xl font-semibold text-light-plum">Payments</h1>
      </div>
      <div className="py-4 relative">
        <div className="bg-gray-accent2 -mx-4 sm:mx-0 sm:rounded-xl  overflow-x-auto">
          <PaymentsList />
        </div>
      </div>
    </div>
  );
};

export default PaymentsPage;
