import sensei from "../../utils/sensei"
import { SenseiConfig } from "@l2-technology/sensei-client"

const getConfig = async (): Promise<SenseiConfig> => {
    return sensei.getConfig()
}

export default getConfig