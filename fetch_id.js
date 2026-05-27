fetch('https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=30122')
  .then(r => r.text())
  .then(html => {
    const match = html.match(/dmm\.co\.jp[^"'\s]+/gi);
    console.log('DLsite URL:', match);
  })
