import { Service } from "zkwasm-ts-server";

const service = new Service(()=>{return;}, ()=>{return});
service.initialize();
service.serve();


