import { spawnSync } from 'node:child_process';
const result=spawnSync('npm.cmd',['run','build'],{stdio:'inherit',shell:true,env:{...process.env,VITE_VXKEY_TEST_APP:'1'}});
process.exit(result.status??1);
