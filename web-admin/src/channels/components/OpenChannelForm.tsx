import { useNavigate } from "react-router";
import { Form, Input, Select } from "../../components/form";
import { z } from "zod";
import openChannel from "../mutations/openChannel";
import { useSearchParams } from "react-router-dom";

export const OpenChannelInput = z.object({
  node_connection_string: z.string(),
  amt_sats: z.string(),
  pub: z.string(),
});

const OpenChannelForm = () => {
  let navigate = useNavigate();
  let [searchParams, _setSearchParams] = useSearchParams();
  let initialConnectionString = searchParams.get("connection") || "";

  let visibilityOptions = [
    { value: "true", text: "Public Channel" },
    { value: "false", text: "Private Channel" },
  ];

  return (
    <Form
      submitText="Open Channel"
      schema={OpenChannelInput}
      initialValues={{
        node_connection_string: initialConnectionString,
        amt_sats: "",
        pub: "true",
      }}
      noticePosition="top"
      layout="default"
      onSubmit={async ({ node_connection_string, amt_sats, pub }) => {
        try {
          await openChannel(
            node_connection_string,
            parseInt(amt_sats, 10),
            pub === "true"
          );
          navigate("/admin/channels");
        } catch (e) {
          // TODO: handle error
        }
      }}
    >
      <Input
        autoFocus
        label="Node Connection Info (pubkey@host:port)"
        name="node_connection_string"
      />
      <Input label="Amount Satoshis" name="amt_sats" />
      <Select
        label="Channel Visibility"
        name="pub"
        options={visibilityOptions}
      />
    </Form>
  );
};

export default OpenChannelForm;
