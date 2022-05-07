import CreateInvoiceForm from "../components/CreateInvoiceForm";
import PaymentsList from "../components/PaymentsList";
const ReceiveMoneyPage = () => {
  return (
    <div className="py-6">
      <div className="">
        <h1 className="text-2xl font-semibold text-light-plum">
          Receive Money
        </h1>
      </div>
      <div className="">
        <div className="py-4">
          <div className="bg-plum-100 text-light-plum shadow p-4 rounded-xl">
            <CreateInvoiceForm />
          </div>
        </div>
      </div>

      <div className="mt-8">
        <h1 className="text-2xl font-semibold text-light-plum">
          Incoming Payments
        </h1>
      </div>
      <div className="py-4 relative">
        <div className="bg-gray-accent2 -mx-4 sm:mx-0 sm:rounded-xl  overflow-x-auto">
          <PaymentsList origin="incoming" />
        </div>
      </div>
    </div>
  );
};

export default ReceiveMoneyPage;
