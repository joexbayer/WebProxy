let toggle;
let ip;
let time = new Date();

var loader = '[[@;;;loader;https://upload.wikimedia.org/wikipedia/commons/1/14/Loading2.gif]Loader Animation]';

jQuery(document).ready(async function($, undefined) {

    var term = $('#terminal').terminal( async function(command) {
        if(command == "help"){
          this.echo(" Commands: ");
          this.echo(" 'connect' => Connect to proxy with chosen IP address.");
          this.echo(" 'disconnect' => Disconnect from proxy.");
          this.echo(" 'set ip <ip>' => Sets IP that proxy will connect too.");
          this.echo(" 'ip' => Prints out current set IP address.");
          this.echo(" 'status' => Shows current status of connection");
          return;
        }

        if(command == "connect"){
          document.getElementById("power").src = "/images/power-pending.png";
          let res = $.fn.start_proxy();
          this.echo("Connecting to proxy...");
          var xhr = $.ajax({
            url: "http://ip-api.com/json/"+ip,
            timeout: 3000,
            success : function(data){
              
              document.getElementById("power").src = "/images/power.png";
              term.echo("Connected to "+data["country"]);
              term.echo("Region: "+data["region"]);
              term.echo("City: "+data["city"]);
              term.echo("ISP: "+data["isp"]);
              time = new Date();
            },
            error : function(e){
              console.log(e);
              $.fn.stop_proxy();
              term.echo("Connection failed.");
            }
          });
          return;
        }

        if(command == "status"){
          if(toggle){
            this.echo("Status: Connected");
            this.echo("IP: "+ip);
            this.echo("Active since: "+time.toLocaleTimeString());
          } else {
            this.echo("Status: Disconnected");
            this.echo("Last active: "+time.toLocaleTimeString());
          }
          return;
        }

        if(command == "disconnect"){
          $.fn.stop_proxy();
          this.echo("Disconnected from proxy.");
          return;
        }

        if(command == "ip"){
          this.echo(ip);
          return;
        }

        if(String(command).includes("set ip")){
          let ip = String(command).split(" ")[2];
          $.fn.updateIp(ip);
          return;
        }

        this.echo("Use 'help' for a overview of possible commands!");
    }, {
        greetings: "Proxy Terminal - use 'help' for commands.",
        name: 'Proxy Terminal',
        prompt: 'vpn> '
    });

    $.fn.stop_proxy = function(){
      document.getElementById("power").src = "/images/power-on.png";
      let config = {
        mode: "fixed_servers",
        rules: {
            singleProxy: {
              scheme: "http",
              host: ip,
              port:12345
            },
            bypassList: ["*"]
          }
      };
      toggle = !toggle;
      chrome.storage.local.set({"state": false}, function() {});
      chrome.proxy.settings.set(
        {value: config, scope: 'regular'},
        function() {});
    }

    $.fn.start_proxy = async function(){
      let config = {
        mode: "fixed_servers",
        rules: {
            singleProxy: {
              scheme: "http",
              host: ip.trim(),
              port:12345
            },
            bypassList: [""]
          }
      };
      toggle = !toggle;
      chrome.storage.local.set({"state": true}, function() {});
      chrome.proxy.settings.set(
        {value: config, scope: 'regular'},
        function() {});
    
    }

  
    $.fn.updateIp = function(ip) {
      chrome.storage.local.set({"proxy_ip": ip}, function() {
        console.log('Value is set to ' + ip);
        ip = ip;
        document.getElementById("ip").innerHTML = ip;
      });
    }

    $.fn.update = function(){
      console.log("running upate");
      chrome.storage.local.get('proxy_ip', function(result) {
        console.log('Value currently is ' + result.proxy_ip);
        ip = result.proxy_ip;
        document.getElementById("ip").innerHTML = ip;
      });
      chrome.storage.local.get('state', function(result) {
        console.log('Value currently is ' + result.state);
        toggle = result.state;
        if(toggle){
          document.getElementById("power").src = "/images/power.png";
        } else {
          document.getElementById("power").src = "/images/power-on.png";
        }
      });

      var xhr = $.ajax({
        url: "https://www.google.com/",
        timeout: 3000,
        success : function(data){
        },
        error : function(e){
          console.log(e);
          $.fn.stop_proxy();
          term.echo("Proxy got disconnected.");
        }
      });
    }
    $.fn.update();
});