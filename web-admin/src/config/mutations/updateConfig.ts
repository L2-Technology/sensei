import sensei from "../../utils/sensei"

const updateConfig = async ({electrumUrl}) => {
    return sensei.updateConfig({electrumUrl})
}

export default updateConfig