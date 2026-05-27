const sqlite3 = require('sqlite3').verbose();
const db = new sqlite3.Database('src-tauri/db/demo.sqlite'); 

db.serialize(() => {
  db.each("SELECT shoukai FROM gamelist WHERE id=30122", (err, row) => {
    console.log('shoukai:', row ? row.shoukai : null);
  });
});
db.close();
