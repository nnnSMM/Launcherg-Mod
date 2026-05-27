fetch('https://dlsoft.dmm.co.jp/detail/nightingale_0005/')
  .then(r => r.text())
  .then(html => {
    const fs = require('fs');
    fs.writeFileSync('30122_fanza.html', html);
    console.log('Saved 30122_fanza.html');
  })
