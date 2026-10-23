import { chromium } from 'playwright-core';
import assert from 'node:assert/strict';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || '/usr/bin/chromium', headless: true, args: ['--no-sandbox'] });
try {
 const page = await browser.newPage({viewport:{width:1440,height:1100}});
 page.setDefaultTimeout(15000);
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto(process.env.APP_URL || 'http://localhost:1420');
 await page.getByRole('heading',{name:'A fresh start for your device.'}).waitFor();
 assert.equal(await page.getByRole('button',{name:'Scan devices'}).isDisabled(),true);
 assert.equal(await page.getByRole('button',{name:'Start flashing'}).isDisabled(),true);
 assert.equal(await page.getByRole('button',{name:'Browse',exact:true}).count(),5);
 await page.getByRole('button',{name:'Connection guide'}).click();
 await page.getByRole('dialog').waitFor();
 await page.getByRole('button',{name:'Got it'}).click();
 assert.equal(await page.getByRole('dialog').count(),0);
 const toggle=page.getByRole('checkbox');assert.equal(await toggle.isChecked(),true);await toggle.uncheck();assert.equal(await toggle.isChecked(),false);
 const download=page.waitForEvent('download');await page.getByRole('button',{name:'Export log'}).click();assert.equal((await download).suggestedFilename(),'odin-operation.log');
 await page.screenshot({path:'/tmp/odin-desktop.png',fullPage:true});
 assert.deepEqual(errors,[]);
 const native = await browser.newPage();
 native.setDefaultTimeout(15000);
 await native.addInitScript(() => {
   window.isTauri = true;
   window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
   window.__calls = [];
   window.__TAURI_INTERNALS__ = {
     transformCallback: () => 1,
     unregisterCallback: () => {},
     invoke: async (cmd, args) => {
       window.__calls.push({cmd,args});
       if(cmd === 'inspect_engine') return 'Odin4 test engine';
       if(cmd === 'scan_devices') return ['/dev/bus/usb/001/002'];
       if(cmd === 'plugin:dialog|open') return '/tmp/AP_test.tar.md5';
       if(cmd === 'plugin:event|listen') return 1;
       if(cmd === 'plugin:event|unlisten') return null;
       if(cmd === 'run_operation') return null;
       throw new Error('Unexpected command: '+cmd);
     }
   };
 });
 await native.goto(process.env.APP_URL || 'http://localhost:1420');
 await native.getByRole('button',{name:'Scan devices'}).click();
 await native.getByRole('heading',{name:'Samsung device connected'}).waitFor();
 await native.getByRole('button',{name:'Browse',exact:true}).nth(1).click();
 assert.equal(await native.getByRole('button',{name:'Start flashing'}).isDisabled(),true);
 await native.getByRole('button',{name:'Validate firmware'}).click();
 await native.getByText('Selected files passed Odin4 checks.').waitFor();
 await native.getByRole('button',{name:'Start flashing'}).click();
 assert.equal(await native.getByRole('button',{name:'Confirm & flash firmware'}).isDisabled(),true);
 await native.getByRole('checkbox',{name:'I have backed up'}).check();
 await native.getByRole('button',{name:'Confirm & flash firmware'}).click();
 await native.getByText('Flash completed',{exact:true}).waitFor();
 const calls=await native.evaluate(()=>window.__calls.filter(c=>c.cmd==='run_operation'));
 assert.equal(calls.length,2);assert.equal(calls[0].args.flash,false);assert.equal(calls[1].args.flash,true);assert.equal(calls[1].args.confirmed,true);
 assert.equal(await native.getByRole('button',{name:'Start flashing'}).isDisabled(),true);
 console.log('PASS: mocked desktop scan, file selection, validation gating, explicit confirmation and flash completion.');
 console.log('PASS: preview renders; browser USB controls disabled; guide, settings, and log export work; no browser errors.');
} finally {await browser.close();}
