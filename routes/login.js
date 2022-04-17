var express = require('express');
var router = express.Router();

/* GET users listing. */
router.get('/', function(req, res, next) {
  var data = {
    title: "Login"
  };
  res.render('./account/login.ejs', data);
});

module.exports = router;
