import { Service } from "zkwasm-ts-server";

const service = new Service(()=>{return;});
service.initialize();
service.serve();


