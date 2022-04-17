//  /mypage/パスのルーター
const router = require("express").Router();
const db = require('../models/index.js');

router.get('/', (req, res, next) => {
  db.Details.findAll().then(usrs => {
    var data = {
      title: 'Mypage',
      content: usrs
    }
    res.render("./account/index.ejs", data);
  });
});

module.exports = router;