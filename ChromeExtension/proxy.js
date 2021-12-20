document.getElementById("onProxy").addEventListener("click", function(){
  var config = {
  mode: "fixed_servers",
  rules: {
      singleProxy: {
        scheme: "http",
        host: "127.0.0.1",
        port:12345
      },
      bypassList: [""]
    }
  };

  chrome.proxy.settings.set(
    {value: config, scope: 'regular'},
    function() {console.log("Loaded!")});
});

document.getElementById("offProxy").addEventListener("click", function(){
  var config = {
  mode: "fixed_servers",
  rules: {
      singleProxy: {
        scheme: "http",
        host: "127.0.0.1",
        port:12345
      },
      bypassList: ["*"]
    }
  };

  chrome.proxy.settings.set(
    {value: config, scope: 'regular'},
    function() {console.log("Removed!")});
});

jQuery(function($, undefined) {
    $('body').terminal(function(command) {
        this.echo("use help");
    }, {
        greetings: 'Proxy Terminal',
        name: 'Proxy Terminal',
        prompt: 'vpn> '
    });
});