const sqlite3 = require('sqlite3').verbose();
const fs = require('fs');
const path = require('path');

const dbPath = path.join(__dirname, 'launcherg_sqlite.db3');
const migrationsDir = path.join(__dirname, 'src', 'migrations');

console.log(`Creating database at ${dbPath}`);
const db = new sqlite3.Database(dbPath);

db.serialize(() => {
    // Get all migration files
    const files = fs.readdirSync(migrationsDir)
        .filter(file => file.endsWith('.sql'))
        .sort(); // Ensure order

    console.log('Found migrations:', files);

    files.forEach(file => {
        const filePath = path.join(migrationsDir, file);
        const sql = fs.readFileSync(filePath, 'utf8');
        console.log(`Applying migration: ${file}`);
        db.exec(sql, (err) => {
            if (err) {
                console.error(`Error applying ${file}:`, err);
                process.exit(1);
            }
        });
    });

    console.log('All migrations applied successfully.');
});

db.close();
