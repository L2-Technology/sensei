import { useNavigate } from "react-router";
import { Form, Input, Select } from "../../components/form";
import { z } from "zod";
import addKnownPeer from "../mutations/addKnownPeer";

export const AddKnownPeerInput = z.object({
  pubkey: z.string(),
  label: z.string(),
  zeroConf: z.string()
});

const AddKnownPeerForm = () => {
  let navigate = useNavigate();

  let zeroConfOptions = [
    { value: "true", text: "Accept 0-Conf" },
    { value: "false", text: "Require Confirmations" },
  ];

  return (
    <Form
      submitText="Add Peer"
      schema={AddKnownPeerInput}
      initialValues={{
        pubkey: "",
        label: "",
        zeroConf: "false",
      }}
      noticePosition="top"
      layout="default"
      onSubmit={async ({ pubkey, label, zeroConf }) => {
        try {
          await addKnownPeer(
            pubkey,
            label,
            zeroConf === "true"
          );
          navigate("/peers");
        } catch (e) {
          // TODO: handle error
        }
      }}
    >
      <Input
        autoFocus
        label="Pubkey"
        name="pubkey"
      />
      <Input label="Label" name="label" />
      <Select
        label="Inbound Channel Requests"
        name="zeroConf"
        options={zeroConfOptions}
      />
    </Form>
  );
};

export default AddKnownPeerForm;
