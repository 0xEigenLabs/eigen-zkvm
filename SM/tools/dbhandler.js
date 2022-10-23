var dbhandler = {
  get: function(target, key) {
    return target[key]
  },
  set: function(target, key, value) {
    target[key] = value;
    return true;
  }
};

module.exports = dbhandler;
