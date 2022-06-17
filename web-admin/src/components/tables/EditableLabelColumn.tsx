import { CheckIcon, PencilAltIcon } from "@heroicons/react/outline";
import { useEffect, useRef, useState } from "react";
import { useQueryClient } from "react-query";

const EditLabelForm = ({ currentLabel, queryKey, updateLabel, setEditing, editing }) => {
  const labelInputEl = useRef(null);
  let queryClient = useQueryClient();
  let [label, setLabel] = useState(currentLabel || "");

  useEffect(() => {
    if(editing && labelInputEl.current) {
      labelInputEl.current.focus()
    }
  }, [editing])

  async function handleSubmit() {
    try {
      if(label !== currentLabel) {
        await updateLabel(label)
      }
      setEditing(false);
      queryClient.invalidateQueries(queryKey);
    } catch (e) {
      // TODO: handle error
    }
  }

  return (
    <div className="flex align-middle items-center">
      <input
        type="text"
        ref={labelInputEl}
        value={label}
        onKeyPress={(e) => {
          if (e.key === "Enter") {
            handleSubmit();
          }
        }}
        name="label"
        className="input"
        onChange={(e) => {
          setLabel(e.target.value);
        }}
      />
      <CheckIcon
        onClick={handleSubmit}
        className="inline-block w-8 h-8 text-green-600 cursor-pointer"
      />
    </div>
  );
};

const EditableLabelColumn = ({ updateLabel, queryKey, label }) => {
  let [editing, setEditing] = useState(false);

  return editing ? (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum`}
    >
      <EditLabelForm 
        currentLabel={label} 
        queryKey={queryKey} 
        updateLabel={updateLabel} 
        editing={editing} 
        setEditing={setEditing} 
      />
    </td>
  ) : (
    <td
      onClick={() => setEditing(true)}
      className={`group cursor-pointer p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum`}
    >
      {label}{" "}
      <span className="inline-block group-hover:hidden">
        &nbsp;&nbsp;&nbsp;&nbsp;
      </span>
      <PencilAltIcon className="w-4 h-4 cursor-pointer hidden group-hover:inline-block" />{" "}
    </td>
  );
}

export default EditableLabelColumn