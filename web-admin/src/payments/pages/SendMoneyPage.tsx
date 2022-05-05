import PayInvoiceForm from "../components/PayInvoiceForm";
import PaymentsList from "../components/PaymentsList";
const SendMoneyPage = () => {
  return (
    <div className="py-6">
      <div className="">
        <h1 className="text-2xl font-semibold text-light-plum">Send Money</h1>
      </div>
      <div className="">
        <div className="py-4">
          <div className="bg-plum-100 shadow p-4 rounded-xl">
            <PayInvoiceForm />
          </div>
        </div>
      </div>

      <div className="mt-8 ">
        <h1 className="text-2xl font-semibold text-light-plum">
          Outgoing Payments
        </h1>
      </div>
      <div className="py-4 relative">
        <div className="bg-gray-accent2 -mx-4 sm:mx-0 sm:rounded-xl overflow-x-auto">
          <PaymentsList origin="outgoing" />
        </div>
      </div>
    </div>
  );
};

export default SendMoneyPage;
