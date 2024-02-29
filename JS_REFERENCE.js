// Define Angular application:
var app = angular.module('OrbitDiagramApp', [ 'ngSanitize', 'ui.bootstrap' ]);

// Configuration:
app.config( function($interpolateProvider) {
  $interpolateProvider.startSymbol('[[');
  $interpolateProvider.endSymbol(']]');
});
// app.config(
//   function ($routeProvider) {
//     $routeProvider
//       .when('/', {})
//       .otherwise({ redirectTo: '/' });
//   });
app.config(
  function($locationProvider) {
    $locationProvider.hashPrefix('');
  });
// We need to allow the orbit-viewer URL (as well as our own URL):
var whitelist = [ 'self' ];
app.config(
  function($sceDelegateProvider) {
    $sceDelegateProvider.resourceUrlWhitelist([
      'self'
    ]);
  });

window.addEventListener('beforeunload', function (e) {
  //
  // Check if data will be lost when reloading:
  // Cancel the event
  e.preventDefault(); // If you prevent default behavior in Mozilla Firefox prompt will always be shown
  // Chrome requires returnValue to be set
  e.returnValue = '';
});

/*==================================================================================================
  GLOBAL VARIABLE DECLARATION

  Define global variables and pre-allocate memory.
*/

var rad2deg = 57.2957795130823;
var deg2rad = 1. / rad2deg;
var rmax = 55;
var gm = 0.00029591220828559;
var au2km = 149597870.700;
var km2au = 1. / au2km;
var day2sec = 86400;

var planet_list = ['Mercury', 'Venus', 'Earth', 'Mars', 'Jupiter', 'Saturn', 'Uranus', 'Neptune', 'Pluto'];
var planet_show = [];
  //
  // Admissible values:
  var validin = {
    qr: [0.01, rmax * .95],
    ecc: [0, 10],
    inc: [0, 180],
    raan: [0, 360],
    omega: [0, 360],
    tp: [2300000, 2600000],
    date: [2300000, 2600000]
  };
var max_range = 50;

var gl_camera = [0, 0];
var gl_maxc;
var should_update = true;
var caldate;

var pl_color = ['gold', 'hotpink', 'limegreen', 'orange', 'red', 'dimgray', 'mediumspringgreen', 'black', 'mediumorchid']

/*==================================================================================================

                                    FUNCTION DEFINITIONS

==================================================================================================*/


//==================================================================================================
//  MAIN CONTROLLER
app.controller('MainController', function ($scope, $location, $window, $timeout) {
  //
  "use strict";
  //
  $scope.main = true;
  $scope.or_controls = true;
  $scope.intro = false;
  //
  // Deactivate loading spinner:
  $('#od_init_loading_spinner').remove();
  //
  $scope.navod_diagram = function() {
    $scope.main = true;
    $scope.intro = false;
  };
  $scope.navod_intro = function() {
    $scope.main = false;
    $scope.intro = true;
  };
  //
  // Launch tool:
  setTimeout(function(){ load_orbit_diagram(); }, 1000);
});

/*==================================================================================================
  function: load_orbit_diagram
*/
function load_orbit_diagram() {
  $(function () {
    $('[data-toggle="tooltip"]').tooltip()
  });
  //
  // Set todays date:
  var today = new Date();
  var dd = String(today.getDate()).padStart(2, '0');
  var mm = String(today.getMonth() + 1).padStart(2, '0');
  var yyyy = today.getFullYear();
  var jd = cal2jd({year: yyyy, month: mm, day: dd});
  $('#inp_date').val(jd['jd'].toFixed(1));
  //
  // Retrieve elements:
  var ids = ['qr', 'ecc', 'inc', 'raan', 'omega', 'tp', 'date'];
  var elem = [];
  for ( var j = 0; j < 6; j++ ) {
    elem.push(parseFloat($('#inp_' + ids[j]).val()));
  }
  if ( $('#or_units').val() == 'km' ) { elem[0] *= km2au; }
  //
  // Get the requested date:
  var date = parseFloat($('#inp_date').val());
  //
  // Propagate orbit:
  var data = get_orbit(elem, date);
  data.elem = elem;
  //
  // Get line of nodes:
  data.node_line = line_of_nodes(elem);
  //
  // Get angular momentum vector:
  data.angmom_vector = angular_momentum_vector(elem);
  //
  // Get eccentricity vector:
  data.ecc_vector = eccentricity_vector(elem);
  //
  plot_orbit(data);
  update_elements();
  //
  // Add listeners:
  var i;
  var Ntrace = 4 + 2 * planet_list.length + 4 + 1;
  for ( i = 0; i < 7; i++ ) {
    $('#inp_' + ids[i]).on('change', function() {
      //
      var un = '';
      //
      // Fetch name:
      var nam = this.id.split('_')[1];
      var val = parseFloat(this.value);
      if ( nam == 'qr' ) {
        if ( $('#or_units').val() == 'km' ) { val *= km2au; }
        un = ' au';
      }
      //
      // Validate input:
      $('.input-error-inp_' + nam).prop('hidden', true);
      $('#inp_' + nam).removeClass('tools-error-input');
      if ( val > validin[nam][1] ) {
        $('.input-error-inp_' + nam).prop('hidden', false);
        $('.input-error-inp_' + nam).html('<b>ERROR:</b> must be <= ' + validin[nam][1] + un);
        $('#inp_' + nam).addClass('tools-error-input');
        return {}
      }
      if ( val < validin[nam][0] ) {
        $('.input-error-inp_' + nam).prop('hidden', false);
        $('.input-error-inp_' + nam).html('<b>ERROR:</b> must be >= ' + validin[nam][0] + un);
        $('#inp_' + nam).addClass('tools-error-input');
        return {}
      }
      //
      // Retrieve elements:
      var elem = [];
      for ( var j = 0; j < 6; j++ ) {
        elem.push(parseFloat($('#inp_' + ids[j]).val()));
      }
      if ( $('#or_units').val() == 'km' ) { elem[0] *= km2au; }
      //
      // Get the requested date:
      var date = parseFloat($('#inp_date').val());
      //
      $('#orbit_note').prop('hidden', true);
      var data = get_orbit(elem, date);
      data.node_line = line_of_nodes(elem);
      data.angmom_vector = angular_momentum_vector(elem);
      data.ecc_vector = eccentricity_vector(elem);
      //
      scale_normal = elem[0] * elem[0] / gm / (1 - elem[1] * elem[1]) * 0.67;
      if ( elem[1] >= 1 || elem[0] * elem[0] / gm / (1 - elem[1]) > data['maxc'] / 3 ) {
        scale_normal = data['maxc'] / 3;
      }
      //
      var update0 = {
        x: [data.x],
        y: [data.y],
        z: [data.z],
        text: [data.text]
      };
      var update1 = {
        x: [[data.ref[0]]],
        y: [[data.ref[1]]],
        z: [[data.ref[2]]],
        text: [data.txtref]
      };
      var update2 = {
        x: [[0, data.node_line.x[0]]],
        y: [[0, data.node_line.y[0]]],
        z: [[0, data.node_line.z[0]]],
      };
      var update3 = {
        x: [[0, data.node_line.x[2]]],
        y: [[0, data.node_line.y[2]]],
        z: [[0, data.node_line.z[2]]],
      };
      var updateN3 = {
        x: [[0, data.angmom_vector.x * scale_normal]],
        y: [[0, data.angmom_vector.y * scale_normal]],
        z: [[0, data.angmom_vector.z * scale_normal]],
      };
      var updateN2 = {
        x: [[data.angmom_vector.x * scale_normal]],
        y: [[data.angmom_vector.y * scale_normal]],
        z: [[data.angmom_vector.z * scale_normal]],
      };
      var updateN1 = {
        x: [[0, data.ecc_vector.x]],
        y: [[0, data.ecc_vector.y]],
        z: [[0, data.ecc_vector.z]],
      };
      var updateN = {
        x: [[data.ecc_vector.x]],
        y: [[data.ecc_vector.y]],
        z: [[data.ecc_vector.z]],
      };
      //
      Plotly.restyle('orb_plot_state', update0, [0]);
      Plotly.restyle('orb_plot_state', update1, [1]);
      Plotly.restyle('orb_plot_state', update2, [2]);
      Plotly.restyle('orb_plot_state', update3, [3]);
      Plotly.restyle('orb_plot_state', updateN3, [Ntrace - 4]);
      Plotly.restyle('orb_plot_state', updateN2, [Ntrace - 3]);
      Plotly.restyle('orb_plot_state', updateN1, [Ntrace - 2]);
      Plotly.restyle('orb_plot_state', updateN, [Ntrace - 1]);
      //
      // Check planet visibility:
      planet_visibility(data['maxc'], date, nam);
      //
      // Update layout:
      var layout = {
        'scene.xaxis.range': [-data['maxc'], data['maxc']],
        'scene.yaxis.range': [-data['maxc'], data['maxc']],
        'scene.zaxis.range': [-data['maxc'], data['maxc']],
      }
      if ( nam == 'date' ) {
        caldate = jd2cal(date);
        layout['title.text'] =
        Plotly.relayout('orb_plot_state', {'title.text': 'State on ' + caldate.string
          + " | Camera angle (" + gl_camera[0].toFixed(0) + ", " + gl_camera[1].toFixed(0) + ") deg"});;
      }
      Plotly.relayout('orb_plot_state', layout);
      update_elements();
    });
  }
  //
  // Units listener:
  $('#or_units').on('change', function(){
    if ( this.value == 'km' ) {
      $('#inp_qr').val(parseFloat($('#inp_qr').val()) * au2km);
      $('#inp_qr').prop('step', 10000000);
    }
    else if ( this.value == 'au' ) {
      $('#inp_qr').val(parseFloat($('#inp_qr').val()) * km2au);
      $('#inp_qr').prop('step', 0.1);
    }
  });
  //
  // Time stepper:
  $('#or_tstep').on('change', function(){
    $('#inp_date').prop('step', this.value);
  });
  //
  // Show vectors:
  $('#check_angmom').on('change', function(){
    Plotly.restyle('orb_plot_state', {visible: this.checked}, [Ntrace - 4, Ntrace - 3]);
  });
  $('#check_eccvec').on('change', function(){
    Plotly.restyle('orb_plot_state', {visible: this.checked}, [Ntrace - 2, Ntrace - 1]);
  });
  //
  // Axis control:
  $('#check_axis_lim').on('change', function(){
    $('.input-error-axis_lim').prop('hidden', true);
    $('#axis_lim').removeClass('tools-error-input');
    if ( this.checked ) {
      $('#axis_lim').prop('disabled', false);
      $('#axis_lim').prop('style', 'width: 10em !important; color: black;');
      if ( $('#axis_lim').val() > rmax ) {
        $('.input-error-axis_lim').prop('hidden', false);
        $('.input-error-axis_lim').html('<b>ERROR:</b> must be <= ' + rmax + ' au');
        $('#axis_lim').addClass('tools-error-input');
        return;
      }
      else if ( $('#axis_lim').val() < 1 ) {
        $('.input-error-axis_lim').prop('hidden', false);
        $('.input-error-axis_lim').html('<b>ERROR:</b> must be >= 1 au');
        $('#axis_lim').addClass('tools-error-input');
        return;
      }
      //
      // Update layout:
      var val = $('#axis_lim').val();
      var layout = {
        'scene.xaxis.range': [-val, val],
        'scene.yaxis.range': [-val, val],
        'scene.zaxis.range': [-val, val],
      }
      Plotly.relayout('orb_plot_state', layout);
      //
      // Check planet visibility:
      planet_visibility($('#axis_lim').val());
    } else {
      $('#axis_lim').prop('disabled', true);
      $('#axis_lim').prop('style', 'width: 10em !important; color: gray;');
      //
      // Update layout:
      var layout = {
        'scene.xaxis.range': [-gl_maxc, gl_maxc],
        'scene.yaxis.range': [-gl_maxc, gl_maxc],
        'scene.zaxis.range': [-gl_maxc, gl_maxc],
      };
      Plotly.relayout('orb_plot_state', layout);
      //
      // Check planet visibility:
      planet_visibility(gl_maxc);
    }
  });
  //
  // Axis control:
  $('#axis_lim').on('change', function() {
    $('.input-error-axis_lim').prop('hidden', true);
    $('#axis_lim').removeClass('tools-error-input');
    if ( this.value > rmax ) {
      $('.input-error-axis_lim').prop('hidden', false);
      $('.input-error-axis_lim').html('<b>ERROR:</b> must be <= ' + rmax + ' au');
      $('#axis_lim').addClass('tools-error-input');
      return;
    }
    else if ( this.value < 1 ) {
      $('.input-error-axis_lim').prop('hidden', false);
      $('.input-error-axis_lim').html('<b>ERROR:</b> must be >= 1 au');
      $('#axis_lim').addClass('tools-error-input');
      return;
    }
    //
    // Update layout:
    var val = $('#axis_lim').val();
    var layout = {
      'scene.xaxis.range': [-val, val],
      'scene.yaxis.range': [-val, val],
      'scene.zaxis.range': [-val, val],
    }
    Plotly.relayout('orb_plot_state', layout);
    //
    // Check planet visibility:
    planet_visibility($('#axis_lim').val());
  });
};


function planet_visibility(lim, date, nam) {
  if(date === undefined) {
    date = 0;
  }
  if(nam === undefined) {
    nam = 'any';
  }
  //
  // Check planet visibility:
  for ( var i = 0; i < planet_list.length; i++ ) {
    var plelem = osculating_elements(planet_list[i]);
    var updatepl = {};
    //
    // Should show:
    if ( 2 * lim * lim > plelem.r45 * plelem.r45 ) {
      if ( !includes(planet_show, planet_list[i]) ) {
        updatepl['visible'] = true;
        planet_show.push(planet_list[i]);
        Plotly.restyle('orb_plot_state', updatepl, [4 + i * 2 + 1, 4 + i * 2 + 2]);
      }
      if ( nam == 'date' ) {
        // var datapl = get_orbit(pplelem, date, true, true);
        var ta = ma2ta(plelem.ec, Math.sqrt(gm / Math.pow(plelem.qr * (1 + plelem.ec), 3))
          * (date - plelem.tp));
        var pplelem = [Math.sqrt(plelem.qr * (1 + plelem.ec) * gm), plelem.ec, plelem.in,
          plelem.om, plelem.w, ta * rad2deg];
        var datapl = elem2cart(pplelem);
        updatepl['x'] = [[datapl['cart'][0]]];
        updatepl['y'] = [[datapl['cart'][1]]];
        updatepl['z'] = [[datapl['cart'][2]]];
        Plotly.restyle('orb_plot_state', updatepl, [4 + i * 2 + 1 + 1]);
      }
    } else {
      updatepl['visible'] = false;
      if ( includes(planet_show, planet_list[i]) ) {
        // planet_show.pop();
        // var idp = planet_show.findIndex(pl => pl === planet_list[i]);
        var idp = -1;
        for ( var ii = 0; ii < planet_show.length; ii++ ) {
          if ( planet_list[i] == planet_show[ii] ) { idp = ii; break; }
        }
        delete planet_show[idp];
        Plotly.restyle('orb_plot_state', updatepl, [4 + i * 2 + 1, 4 + i * 2 + 1 + 1]);
      }
    }
  }
  //
  return;
};


/*==================================================================================================
  function: get_orbit

  Get the states defining the current orbit.
*/
function get_orbit(elem, jd, isplanet, refonly) {
  if(isplanet === undefined) {
    isplanet = false;
  }
  if(refonly === undefined) {
    refonly = false;
  }
  //
  // Store reference TA:
  var reftp = elem[5];
  //
  // Convert to angular momentum:
  var h = Math.sqrt( elem[0] * gm * (1 + elem[1]) );
  //
  var data;
  if ( !refonly ){
    // Get limits in TA:
    var ta_lim = [0, 360];
    if ( elem[1] >= 1 || (elem[1] < 1 && elem[0] / (1 - elem[1]) * (1 + elem[1]) > (rmax * 1.74)) ){
      ta_lim[0] = -Math.acos(1 / elem[1] * (h * h / gm / (rmax * 1.74) - 1)) * rad2deg;
      ta_lim[1] = -ta_lim[0];
      // Display note:
      if ( !isplanet ) {
        $('#orbit_note').prop('hidden', false);
        $('#orbit_note').html('<b>NOTE:</b> the plot is not displaying the entire orbit. The '
          + 'heliocentric distance is limited to ' + rmax.toFixed(0) + ' au.');
      }
    }
    elem[0] = h;
    //
    //
    // Create ta vector:
    elem[5] = ta_lim[0];
    var nta = 360;
    var dta = (ta_lim[1] - ta_lim[0]) / (nta - 1);
    data = {
      x: [],
      y: [],
      z: [],
      vx: [],
      vy: [],
      vz: [],
      text: [],
      txtref: '',
      maxcoord: [0, 0, 0, 0, 0, 0],
      maxc: 0
    };
    for ( var i = 0; i < nta; i++ ) {
      var result = elem2cart(elem, gm);
      data.x.push(result['cart'][0]);
      data.y.push(result['cart'][1]);
      data.z.push(result['cart'][2]);
      data.vx.push(result['cart'][3]);
      data.vy.push(result['cart'][4]);
      data.vz.push(result['cart'][5]);
      elem[5] += dta;
      // var max = Math.max.apply(null, result['cart'].map(Math.abs));
      for ( var j = 0; j < 6; j++ ) {
        if ( Math.abs(result['cart'][j]) > data.maxcoord[j] ) { data.maxcoord[j] = Math.abs(result['cart'][j]); }
      }
    }
  } else {
    elem[0] = h;
    data = {};
  }
  // elem[5] = refta;
  // var result = elem2cart(elem, gm);
  // data['ref'] = result['cart'];
  //
  // Compute reference state:
  elem[5] = 0;
  result = elem2cart(elem, gm);
  var vr0 = [result['cart'][0], result['cart'][1], result['cart'][2]];
  var vv0 = [result['cart'][3], result['cart'][4], result['cart'][5]];
  result = propagateKepler(vr0, vv0, jd - reftp);
  data['ref'] = [];
  for ( var j = 0; j < 3; j++ ) {
    data['ref'][j] = result.pos[j];
    data['ref'][j + 3] = result.vel[j];
  }
  //
  if ( refonly ) { return data; }
  //
  // Max range:
  data['maxc'] = Math.ceil(Math.max(data.maxcoord[0], data.maxcoord[1], data.maxcoord[2]) * 1.1);
  if ( data['maxc'] > rmax ) { data['maxc'] = rmax; }
  if ( !isplanet ) { gl_maxc = data['maxc']; }
  if ( $('#check_axis_lim').is(':checked') ) {
    data['maxc'] = $('#axis_lim').val();
  }
  max_range = data['maxc'];
  //
  // Prepare text:
  for ( var i = 0; i < data.x.length; i++ ) {
    var txt = '<b>User-defined orbit</b>';
    //============================
    txt += '<br><b>X</b>: ' + data.x[i].toExponential(3) + ' au | ' + (data.x[i] * au2km).toExponential(3) + ' km';
    txt += '<br><b>Y</b>: ' + data.y[i].toExponential(3) + ' au | ' + (data.y[i] * au2km).toExponential(3) + ' km';
    txt += '<br><b>Z</b>: ' + data.z[i].toExponential(3) + ' au | ' + (data.z[i] * au2km).toExponential(3) + ' km';
    txt += '<br><b>VX</b>: ' + data.vx[i].toExponential(3) + ' au/d | ' + (data.vx[i] * au2km / day2sec).toExponential(3) + ' km/s';
    txt += '<br><b>VY</b>: ' + data.vy[i].toExponential(3) + ' au/d | ' + (data.vy[i] * au2km / day2sec).toExponential(3) + ' km/s';
    txt += '<br><b>VZ</b>: ' + data.vz[i].toExponential(3) + ' au/d | ' + (data.vz[i] * au2km / day2sec).toExponential(3) + ' km/s';
    //============================
    // txt += '<table style="border: 0px !important;">';
    // txt += '  <tbody>';
    // txt += '    <tr><td><b>x:</b></td><td>' + data.x[i].toExponential(3) + ' au</td><td>' + (data.x[i] * au2km).toExponential(3) + ' km</td></tr>';
    // txt += '    <tr><td><b>y:</b></td><td>' + data.y[i].toExponential(3) + ' au</td><td>' + (data.y[i] * au2km).toExponential(3) + ' km</td></tr>';
    // txt += '    <tr><td><b>z:</b></td><td>' + data.z[i].toExponential(3) + ' au</td><td>' + (data.z[i] * au2km).toExponential(3) + ' km</td></tr>';
    // txt += '    <tr><td><b>vx:</b></td><td>' + data.vx[i].toExponential(3) + ' au/d</td><td>' + (data.vx[i] * au2km / day2sec).toExponential(3) + ' km/s</td></tr>';
    // txt += '    <tr><td><b>vy:</b></td><td>' + data.vy[i].toExponential(3) + ' au/d</td><td>' + (data.vy[i] * au2km / day2sec).toExponential(3) + ' km/s</td></tr>';
    // txt += '    <tr><td><b>vz:</b></td><td>' + data.vz[i].toExponential(3) + ' au/d</td><td>' + (data.vz[i] * au2km / day2sec).toExponential(3) + ' km/s</td></tr>';
    // txt += '  </tbody>';
    // txt += '</table>';
    //============================


    data.text.push(txt);
  }
  data['txtref'] = '<b>User-defined body</b>';
  // data['txtref'] += '<br>x (au): ' + data.ref[0].toExponential(5);
  // data['txtref'] += '<br>y (au): ' + data.ref[1].toExponential(5);
  // data['txtref'] += '<br>z (au): ' + data.ref[2].toExponential(5);
  // data['txtref'] += '<br>vx (au/d): ' + data.ref[3].toExponential(5);
  // data['txtref'] += '<br>vy (au/d): ' + data.ref[4].toExponential(5);
  // data['txtref'] += '<br>vz (au/d): ' + data.ref[5].toExponential(5);
    data['txtref'] += '<br><b>X</b>: ' +  data.ref[0].toExponential(3) + ' au | ' +   (data.ref[0] * au2km).toExponential(3) + ' km';
    data['txtref'] += '<br><b>Y</b>: ' +  data.ref[1].toExponential(3) + ' au | ' +   (data.ref[1] * au2km).toExponential(3) + ' km';
    data['txtref'] += '<br><b>Z</b>: ' +  data.ref[2].toExponential(3) + ' au | ' +   (data.ref[2] * au2km).toExponential(3) + ' km';
    data['txtref'] += '<br><b>VX</b>: ' + data.ref[3].toExponential(3) + ' au/d | ' + (data.ref[3] * au2km / day2sec).toExponential(3) + ' km/s';
    data['txtref'] += '<br><b>VY</b>: ' + data.ref[4].toExponential(3) + ' au/d | ' + (data.ref[4] * au2km / day2sec).toExponential(3) + ' km/s';
    data['txtref'] += '<br><b>VZ</b>: ' + data.ref[5].toExponential(3) + ' au/d | ' + (data.ref[5] * au2km / day2sec).toExponential(3) + ' km/s';
  //
  // Return data:
  return data;
}


/*==================================================================================================
  function: plot_orbit

  Render the custom orbit.
*/
function plot_orbit(data) {
  //
  // Initialize trace:
  var trace = [];
  trace.push({
    type: 'scatter3d',
    mode: 'lines',
    name: 'User-defined orbit',
    x: data.x,
    y: data.y,
    z: data.z,
    text: data.text,
    hoverinfo: 'text',
    line: {
      width: 5,
      // color: 'rgb(29, 106, 169)'
      color: 'cornflowerblue'
    },
    hoverlabel: {
      font: {color: 'white'}
    }
  });
  trace.push({
    type: 'scatter3d',
    mode: 'marker',
    name: 'User-defined body',
    x: [data.ref[0]],
    y: [data.ref[1]],
    z: [data.ref[2]],
    text: data.txtref,
    hoverinfo: 'text',
    marker: {
      color: 'cornflowerblue',
      // color: 'rgb(29, 106, 169)',
      size: 5
    },
    hoverlabel: {
      font: {color: 'white'}
    },
    showlegend: false
  });
  //
  // Plot line of nodes:
  trace.push({
    type: 'scatter3d',
    mode: 'lines',
    name: 'Line of nodes (ascending)',
    x: [0, data.node_line.x[0]],
    y: [0, data.node_line.y[0]],
    z: [0, data.node_line.z[0]],
    hoverinfo: 'none',
    line: {
      width: 2,
      dash: 'solid',
      // color: 'rgb(29, 106, 169)'
      color: 'cornflowerblue'
    },
    hoverlabel: {
      font: {color: 'white'}
    },
    showlegend: false
  });
  //
  // Plot line of nodes:
  trace.push({
    type: 'scatter3d',
    mode: 'lines',
    name: 'Line of nodes (descending)',
    x: [0, data.node_line.x[2]],
    y: [0, data.node_line.y[2]],
    z: [0, data.node_line.z[2]],
    hoverinfo: 'none',
    line: {
      width: 4,
      dash: 'dash',
      // color: 'rgb(29, 106, 169)'
      color: 'cornflowerblue'
    },
    hoverlabel: {
      font: {color: 'white'}
    },
    showlegend: false
  });
  //
  // Add Sun:
  trace.push({
    type: 'scatter3d',
    mode: 'marker',
    name: 'Sun',
    hoverinfo: 'name',
    showlegend: false,
    x: [0],
    y: [0],
    z: [0],
    marker: {
      color: 'orange',
      size: 4
    }
  });
  //
  // Process planets:
  for ( var ipl = 0; ipl < planet_list.length; ipl++ ) {
    var planet_elem = osculating_elements(planet_list[ipl]);
    //
    // Get the requested date:
    var date = parseFloat($('#inp_date').val());
    caldate = jd2cal(date);
    //
    // Get r45:
    var elem_pl = [planet_elem.qr, planet_elem.ec, planet_elem.in, planet_elem.om, planet_elem.w, 0];
    var planet_data = get_orbit(elem_pl, date, true);
    var tr = {
      type: 'scatter3d',
      mode: 'lines',
      hoverinfo: 'text',
      text: planet_list[ipl] + ' orbit',
      name: planet_list[ipl],
      x: planet_data.x,
      y: planet_data.y,
      z: planet_data.z,
      line: {
        width: 2,
        color: pl_color[ipl]
      },
      visible: false,
      showlegend: true
    };
    if ( 2 * data['maxc'] * data['maxc'] >= planet_elem.r45 * planet_elem.r45 ) {
      tr['visible'] = true;
      planet_show.push(planet_list[ipl]);
    }
    trace.push(tr);
    tr = {
      type: 'scatter3d',
      mode: 'marker',
      name: planet_list[ipl],
      x: [planet_data.ref[0]],
      y: [planet_data.ref[1]],
      z: [planet_data.ref[2]],
      hoverinfo: 'name',
      marker: {
        color: pl_color[ipl],
        size: 4
      },
      visible: false,
      showlegend: false
    };
    if ( 2 * data['maxc'] * data['maxc'] >= planet_elem.r45 * planet_elem.r45 ) {
      tr['visible'] = true;
      planet_show.push(planet_list[ipl]);
    }
    trace.push(tr);
  }
  //
  // // Add ecliptic lines:
  // trace.push({
  //   type: 'scatter3d',
  //   mode: 'lines',
  //   hoverinfo: 'none',
  //   x: [-rmax * 1.1, rmax * 1.1],
  //   y: [0, 0],
  //   z: [0, 0],
  //   line: {
  //     width: .5,
  //     color: 'black',
  //     dash: 'dash'
  //   },
  //   showlegend: false
  // });
  // trace.push({
  //   type: 'scatter3d',
  //   mode: 'lines',
  //   hoverinfo: 'none',
  //   x: [0, 0],
  //   y: [-rmax * 1.1, rmax * 1.1],
  //   z: [0, 0],
  //   line: {
  //     width: .5,
  //     color: 'black',
  //     dash: 'dash'
  //   },
  //   showlegend: false
  // });
  //
  // Plot angular momentum vector:
  scale_normal = data.elem[0] * data.elem[0] / gm / (1 - data.elem[1] * data.elem[1]) * 0.67;
  if ( data.elem[1] >= 1 || data.elem[0] * data.elem[0] / gm / (1 - data.elem[1]) > data['maxc'] / 3 ) {
    scale_normal = data['maxc'] / 3;
  }
  trace.push({
    type: 'scatter3d',
    mode: 'lines',
    name: 'Normal to orbital plane',
    x: [0, data.angmom_vector.x * scale_normal],
    y: [0, data.angmom_vector.y * scale_normal],
    z: [0, data.angmom_vector.z * scale_normal],
    hoverinfo: 'none',
    line: {
      width: 2,
      dash: 'solid',
      // color: 'rgb(29, 106, 169)'
      color: 'cornflowerblue'
    },
    hoverlabel: {
      font: {color: 'white'}
    },
    visible: false,
    showlegend: false
  });
  trace.push({
    type: 'scatter3d',
    mode: 'text',
    x: [data.angmom_vector.x * scale_normal],
    y: [data.angmom_vector.y * scale_normal],
    z: [data.angmom_vector.z * scale_normal],
    text: '<b>h</b>',
    textfont: {
      size: 10
    },
    visible: false,
    showlegend: false
  });
  //
  // Plot eccentricity vector:
  scale_ecc = 1;
  trace.push({
    type: 'scatter3d',
    mode: 'lines',
    name: 'Eccentricity vector',
    x: [0, data.ecc_vector.x * scale_ecc],
    y: [0, data.ecc_vector.y * scale_ecc],
    z: [0, data.ecc_vector.z * scale_ecc],
    hoverinfo: 'none',
    line: {
      width: 2,
      dash: 'solid',
      // color: 'rgb(29, 106, 169)'
      color: 'cornflowerblue'
    },
    hoverlabel: {
      font: {color: 'white'}
    },
    visible: false,
    showlegend: false
  });
  trace.push({
    type: 'scatter3d',
    mode: 'text',
    x: [data.ecc_vector.x * scale_ecc],
    y: [data.ecc_vector.y * scale_ecc],
    z: [data.ecc_vector.z * scale_ecc],
    text: '<b>e</b>',
    textfont: {
      size: 10
    },
    visible: false,
    showlegend: false
  });
  //
  gl_camera[0] = 45;
  gl_camera[1] = 45;
  var layout = {
    height: 700,
    width: 700,
    title: {
      text: 'State on ' + caldate.string + " | Camera angle (" + gl_camera[0].toFixed(0) + ", " + gl_camera[1].toFixed(0) + ") deg",
      x: 0.4,
      font: {
        size: 12
      }
    },
    scene:{
      aspectmode: "manual",
      autosize: true,
      aspectratio: {x: 1, y: 1, z: 1},
      camera: {eye: {x: +1.4, y: +1.4, z: +1.4}},
      xaxis: {
        backgroundcolor: 'white',
        autotick: true,
        showbackground: true,
        gridcolor: 'rgb(220, 220, 220)',
        gridwidth: 1.5,
        showgrid: true,
        zerolinecolor: 'rgb(220, 220, 220)',
        zerolinewidth: 5,
        title: 'X<sub>eclip</sub> (au)',
        showspikes: false,
        mirror: false,
        color: 'black',
        showline: true,
        linecolor: 'rgb(220, 220, 220)',
        linewidth: 2,
        range: [-data['maxc'], data['maxc']],
      },
      yaxis: {
        backgroundcolor: 'white',
        autotick: true,
        showbackground: true,
        gridcolor: 'rgb(220, 220, 220)',
        gridwidth: 1.5,
        zerolinecolor: 'rgb(220, 220, 220)',
        zerolinewidth: 5,
        showgrid: true,
        title: 'Y<sub>eclip</sub> (au)',
        showspikes: false,
        mirror: false,
        color: 'black',
        showline: true,
        linecolor: 'rgb(220, 220, 220)',
        linewidth: 2,
        range: [-data['maxc'], data['maxc']],
      },
      zaxis: {
        backgroundcolor: 'white',
        autotick: true,
        showbackground: true,
        gridcolor: 'rgb(220, 220, 220)',
        gridwidth: 1.5,
        zerolinecolor: 'rgb(220, 220, 220)',
        zerolinewidth: 5,
        showgrid: true,
        title: 'Z<sub>eclip</sub> (au)',
        showspikes: false,
        color: 'black',
        mirror: false,
        showline: true,
        linecolor: 'rgb(220, 220, 220)',
        linewidth: 2,
        range: [-data['maxc'], data['maxc']],
      },
    },
    margin: {
      t: 30,
      l: 0
    }
  }
  //
  // Options:
  var options = {
    doubleClick: false,
    showLink: false,
    displaylogo: false,
    // displayModeBar: true,
    editable: false,
    toImageButtonOptions: {
      format: 'png',
      scale: 5,
      filename: 'orbit'
    },
    modeBarButtonsToRemove: [ 'sendDataToCloud', 'resetCameraLastSave3d', 'orbitRotation',
      'hoverClosest3d' ],
  };
  //
  var orbitPlot = document.getElementById('orb_plot_state')
  Plotly.newPlot(orbitPlot, trace, layout, options)
  .then(gd => {
    gd.on('plotly_relayout', function(data) {
      if (data.hasOwnProperty('scene.camera')){
        var cam = data['scene.camera']['eye'];
        var r = Math.sqrt(cam.x * cam.x + cam.y * cam.y + cam.z * cam.z);
        var lat = Math.asin(cam.z / r) * rad2deg;
        var lon = Math.atan2(cam.y, cam.x) * rad2deg;
        gl_camera[0] = lat;
        gl_camera[1] = lon;
        Plotly.relayout('orb_plot_state', {'title.text': 'State on ' + caldate.string
          + " | Camera angle (" + lat.toFixed(0) + ", " + lon.toFixed(0) + ") deg"});
      }
      return;
    })
  });
  orbitPlot.on('plotly_legendclick', function(data){
    console.log(['data', data])
    var id = data.curveNumber;
    var visible = data.data[id].visible;
    if ( visible == true ) {
      Plotly.restyle(orbitPlot, {visible: 'legendonly'}, [id, id + 1]);
    }
    else if ( visible == 'legendonly' ) {
      Plotly.restyle(orbitPlot, {visible: true}, [id, id + 1]);
    }
    return false;
  });
};


/*==================================================================================================
  function: elem2cart

  Convert from elements to state.
*/
function elem2cart(elem) {
  var h = elem[0];
  var ecc = elem[1];
  var inc = elem[2] * deg2rad;
  var raan = elem[3] * deg2rad;
  var peri = elem[4] * deg2rad;
  var ta = elem[5] * deg2rad;
  var long = ta + peri;
  //
  // Compute angular momentum:
  // var h = Math.sqrt( Math.abs(sma * gm * (1 - ecc**2) ));
  //
  // Compute radial distance:
  var r = h * h / gm / (1 + ecc * Math.cos(ta));
  //
  // Compute radial and tangential velocities:
  var vr = gm / h * ecc * Math.sin(ta);
  var vth = gm / h * (1 + ecc * Math.cos(ta));
  //
  // Project radial and tangential unit vectors on the inertial frame:
  var ur =  [ Math.cos(raan) * Math.cos(long) - Math.sin(raan) * Math.cos(inc) * Math.sin(long),
       Math.sin(raan) * Math.cos(long) + Math.cos(raan) * Math.cos(inc) * Math.sin(long),
       Math.sin(inc) * Math.sin(long) ];
  var uth = [ -Math.cos(raan) * Math.sin(long) - Math.sin(raan) * Math.cos(inc) * Math.cos(long),
        -Math.sin(raan) * Math.sin(long) + Math.cos(raan) * Math.cos(inc) * Math.cos(long),
         Math.sin(inc) * Math.cos(long) ];
  //
  // State vector:
  var cart = [];
  for ( var i = 0; i < 3; i++ ) { cart.push(ur[i] * r); }
  for ( var i = 0; i < 3; i++ ) { cart.push(vr * ur[i] + vth * uth[i]); }
  return {cart: cart};
};


function osculating_elements(body) {
  // Osculating elements at JD 2459024.5
  var elem;
  if ( body == 'Mercury' ) {
    elem = {
      ec: 2.056408220896557E-01,
      qr: 3.074958016246215E-01,
      tp: 2459067.650840002578,
      om: 4.830597718083336E+01,
      w: 2.918348714438387E+01,
      in: 7.003733902930839E+00,
    };
  } else if ( body == 'Venus' ) {
    elem = {
      ec: 6.762399503226460E-03,
      qr: 7.184498538218194E-01,
      tp: 2458928.746738597285,
      om: 7.662382745452284E+01,
      w: 5.508424062115083E+01,
      in: 3.394576883214484E+00,
    };
  } else if ( body == 'Earth' ) {
    elem = {
      ec: 1.596622548529253E-02,
      qr: 9.847638666827956E-01,
      tp: 2458852.059663722757,
      om: 1.444234845362845E+02,
      w: 3.181651506357810E+02,
      in: 3.251531504436147E-03,
    };
  } else if ( body == 'Mars' ) {
    elem = {
      ec: 9.345385724812259E-02,
      qr: 1.381380519592450E+00,
      tp: 2459064.867384146899,
      om: 4.949761968250743E+01,
      w: 2.865976300648143E+02,
      in: 1.847890654037755E+00,
    };
  } else if ( body == 'Jupiter' ) {
    elem = {
      ec: 4.859977273897987E-02,
      qr: 4.950293643194364E+00,
      tp: 2459965.784028965980,
      om: 1.005188687941560E+02,
      w: 2.735384345047322E+02,
      in: 1.303874556209337E+00,
    };
  } else if ( body == 'Saturn' ) {
    elem = {
      ec: 5.111420347186896E-02,
      qr: 9.092964078253944E+00,
      tp: 2463550.111982490402,
      om: 1.136005677953501E+02,
      w: 3.370669187238602E+02,
      in: 2.489627626792582E+00,
    };
  } else if ( body == 'Uranus' ) {
    elem = {
      ec: 4.590222901851362E-02,
      qr: 1.830882300588441E+01,
      tp: 2470253.757045442238,
      om: 7.408400460182800E+01,
      w: 9.832413508468296E+01,
      in: 7.705441985590336E-01,
    };
  } else if ( body == 'Neptune' ) {
    elem = {
      ec: 1.098118165219995E-02,
      qr: 2.989413845714193E+01,
      tp: 2463514.590601669159,
      om: 1.318922210631964E+02,
      w: 2.440980588412000E+02,
      in: 1.775000722400165E+00,
    };
  } else if ( body == 'Pluto' ) {
    elem = {
      ec: 2.571893642274664E-01,
      qr: 2.991037668682205E+01,
      tp: 2448287.417436909862,
      om: 1.103222267937496E+02,
      w: 1.169758522861960E+02,
      in: 1.723293551253252E+01,
    };
  }
  elem['r45'] = elem.qr * (1 + elem.ec) / (1 + .5 * Math.sqrt(2.) * elem.ec );
  return elem;
};


function line_of_nodes(elem) {
  var h = elem[0];
  var ecc = elem[1];
  var inc = elem[2] * deg2rad;
  var raan = elem[3] * deg2rad;
  var peri = elem[4] * deg2rad;
  var ta = elem[5] * deg2rad;
  var th1 = raan;
  var th2 = Math.PI + raan;
  //
  // Compute distance to ascending and descending nodes:
  var ra = h * h / gm / (1.0 + ecc * Math.cos(-peri));
  var rd = h * h / gm / (1.0 + ecc * Math.cos(Math.PI - peri));
  console.log(['radii', ra, rd])
  //
  // Adjust open orbits:
  if ( ra < 0 ) {
    ra = 0;
    th1 = th2;
  } else if ( rd < 0 ) {
    rd = 0;
    th2 = th1;
  }
  // if ( ra == Infinity ) {
  if ( ra > max_range ) {
    ra = 0;
    th1 = th2;
  // } else if ( rd == Infinity ) {
  } else if ( rd > max_range ) {
    rd = 0;
    th2 = th1;
  }
  //
  // Project nodes on inertial frame:
  var lon = {'x': [ra * Math.cos(th1), 0, rd * Math.cos(th2)],
    'y': [ra * Math.sin(th1), 0, rd * Math.sin(th2)], 'z': [0, 0, 0]};
  //
  return lon;
};


function angular_momentum_vector(elem) {
  var h = elem[0];
  var ecc = elem[1];
  var inc = elem[2] * deg2rad;
  var raan = elem[3] * deg2rad;
  var peri = elem[4] * deg2rad;
  var ta = elem[5] * deg2rad;
  //
  // Project direction of angular momentum vector:
  return {x: Math.sin(raan) * Math.sin(inc), y: -Math.cos(raan) * Math.sin(inc), z: Math.cos(inc)};
};


function eccentricity_vector(elem) {
  var h = elem[0];
  var ecc = elem[1];
  var inc = elem[2] * deg2rad;
  var raan = elem[3] * deg2rad;
  var peri = elem[4] * deg2rad;
  var ta = elem[5] * deg2rad;
  //
  // Project direction of eccentricity vector:
  var rp = h * h / gm / (1 + ecc);
  return {x: rp * (Math.cos(raan) * Math.cos(peri) - Math.sin(raan) * Math.cos(inc) * Math.sin(peri)),
          y: rp * (Math.sin(raan) * Math.cos(peri) + Math.cos(raan) * Math.cos(inc) * Math.sin(peri)),
          z: rp * Math.sin(peri) * Math.sin(inc)
  }
};

// Convert MJD to calendar YYYY-MM-DD: does not correct for the Gregorian shift
function jd2cal( jd ) {
  // var jd = parseFloat(mjd) + 2400000.5;
  var Q = jd + 0.5;
  var Z = Math.floor( Q );
  // var fod = Q - Z;
  var W = Math.floor( ( Z - 1867216.25 ) * 0.273790700698850763533816573920e-4 );
  var X = Math.floor( W * .25 );
  var A = Z + 1 + W - X;
  var B = A + 1524;
  var C = Math.floor( ( B - 122.1 ) * 0.273785078713210130047912388775e-2 );
  var D = Math.floor( 365.25 * C );
  var E = Math.floor( ( B - D ) * 0.326796317659092617344387763439e-1 );
  var F = Math.floor( 30.6001 * E );
  var day = Math.floor( B - D - F + ( Q - Z ) );
  var month = E - 1;
  if ( (month > 12) || (month < 1) ) {
    month = E - 13;
  }
  var year;
  if ( ( month == 1 ) || ( month == 2 ) ) {
    year = C - 4715;
  } else {
    year = C - 4716;
  }
  // var fod_s = fod * 86400;
  // var hour = Math.floor( fod_s / 3600 );
  // var min = Math.floor( ( fod_s - hour * 3600 ) / 60 );
  // var sec = fod_s - hour * 3600 - min * 60;

  var formatted = year.toFixed(0) + "-" + padding(month, 2) + "-" + padding(day, 2);

  return {day: day, month: month, year: year, string: formatted};
};  // End of mjd2cal

// Add padding with a function:
function padding( num, size ) {
  var s = String(num);
  while (s.length < (size || 2)) {s = "0" + s;}
  return s;
};  // End of padding

// Add padding with a method:
Number.prototype.pad = function( size ) {
  var s = String(this);
  while (s.length < (size || 2)) {s = "0" + s;}
  return s;
};  // End of pad


// Get scope outside the controller:
function getScope(ctrlName) {
    var sel = 'div[ng-controller="' + ctrlName + '"]';
    return angular.element(sel).scope();
};

// Convert calendar date YYYY-MM-DD to JD
function cal2jd( cal ) {
  // Number of milliseconds from 1970-Jan-01 00:00:00 UT
  var diff = Date.UTC( cal.year, cal.month - 1, cal.day );
  // Convert to days:
  diff = diff / 86400000;
  // Get MJD:
  return {jd: diff + 40587 + 2400000.5};
}; // End of mjd2cal

if (!String.prototype.padStart) {
    String.prototype.padStart = function padStart(targetLength,padString) {
        targetLength = targetLength>>0; //truncate if number or convert non-number to 0;
        padString = String((typeof padString !== 'undefined' ? padString : ' '));
        if (this.length > targetLength) {
            return String(this);
        }
        else {
            targetLength = targetLength-this.length;
            if (targetLength > padString.length) {
                padString += padString.repeat(targetLength/padString.length); //append to original to ensure we are longer than needed
            }
            return padString.slice(0,targetLength) + String(this);
        }
    };
};


/*==================================================================================================
  FUNCTION: propagateKepler

  Propagate one step of a Keplerian orbit.
*/
function propagateKepler(vr0, vv0, dt) {
  //
  // Tolerance
  var tol = 1e-12;
  //
  // Compute vector norms:
  var r0 = norm(vr0);
  var v0 = norm(vv0);
  //
  // Compute energy and derived quantities:
  var xi = v0 * v0 * .5 - gm / r0;
  var sma = -gm / ( 2 * xi );
  var alpha = 1.0 / sma;
  var chi0;
  if ( alpha > 0.000001 ) {
    chi0 = Math.sqrt( gm ) * dt * alpha;
  } else if (alpha<-0.000001) {
    chi0 = Math.sign( dt ) * Math.sqrt( -sma )
      * Math.log( -2 * gm * alpha * dt / ( dot( vr0, vv0 )
      + Math.sign( dt ) * Math.sqrt( -gm * sma ) * ( 1 - r0 * alpha) ) );
  } else {
    var vh = cross( vr0, vv0 );
    var p = Math.pow( norm( vh ) , 2 ) / gm;
    var s = .5 * Math.atan( 1.0 / ( 3 * Math.sqrt( gm / ( p * p * p ) ) * dt ) );
    var w = Math.atan( Math.pow( Math.tan( s ) , 0.333333333333333333 ) );
    chi0 = Math.sqrt( p ) * 2 / Math.tan( 2 * w );
  }
  //
  var r, chi, c2, c3, psi;
  for ( var j = 0; j < 500; j++ ) {
    psi = chi0 * chi0 * alpha;
    var result = find_c2c3( psi, tol );
    c2 = result.c2;
    c3 = result.c3;
    r = chi0 * chi0 * c2 + dot( vr0, vv0 ) /
      Math.sqrt( gm ) * chi0 * ( 1 - psi * c3 ) + r0 * ( 1 - psi * c2 );
    chi = chi0 + ( Math.sqrt( gm ) * dt - chi0 * chi0 * chi0 * c3 - dot( vr0, vv0 )
      / Math.sqrt( gm ) * chi0 * chi0 * c2 - r0 * chi0 * ( 1 - psi * c3 ) ) / r;
    if ( Math.abs( chi - chi0 ) < tol ) { break; }
    chi0 = chi;
  }
  //
  // Compute f and g functions:
  var f  = 1 - chi * chi / r0 * c2;
  var g  = dt - chi * chi * chi / Math.sqrt( gm ) * c3;
  var dg = 1 - chi * chi / r * c2;
  var df = Math.sqrt( gm ) / ( r * r0 ) * chi * ( psi * c3 - 1 );
  //
  // Compute state vector:
  var vr = vr0.map(function (num, idx) { return num * f + vv0[idx] * g } );
  var vv = vr0.map(function (num, idx) { return num * df + vv0[idx] * dg } );
  //
  return { pos: vr, vel: vv };
};  // End of propagateKepler


function update_elements() {
  var qr = parseFloat($('#inp_qr').val());
  var ecc = parseFloat($('#inp_ecc').val());
  var inc = parseFloat($('#inp_inc').val());
  var raan = parseFloat($('#inp_raan').val());
  var omega = parseFloat($('#inp_omega').val());
  var tp = parseFloat($('#inp_tp').val());
  //
  // Derived variables:
  var type;
  var sma = qr / ( 1 - ecc );
  var period = 2 * Math.PI * Math.sqrt( sma * sma * sma / gm );
  var n = Math.sqrt( gm / sma / sma / sma );
  var vper = Math.sqrt( 2 * gm / qr - gm / sma ) * au2km / day2sec;
  var vapo = Math.sqrt( 2 * gm / (sma * (1 + ecc)) - gm / sma ) * au2km / day2sec;
  sma = sma.toFixed(2);
  period = period.toFixed(2);
  n = n.toExponential(4);
  vper = vper.toFixed(2);
  vapo = vapo.toFixed(2);
  if ( ecc == 0 ) {
    type = "Circular";
  } else if ( ecc < 1 ) {
    type = "Elliptic";
  } else if ( ecc == 1 ) {
    type = "Parabolic";
    sma = "Inf";
    period = "N/A";
    n = "N/A";
    vapo = "N/A";
  } else if ( ecc > 1 ) {
    type = "Hyperbolic";
    period = "N/A";
    n = "N/A";
    vapo = "N/A";
  }
  var date = jd2cal(tp);
  $("#orbit_class").html(type);
  $("#orbital_period").html(period);
  $("#mean_motion").html(n);
  $("#peri_date").html(date.string);
  $("#peri_vel").html(vper);
  $("#apo_vel").html(vapo);
  $("#semimajor_axis").html(sma);
  return {};
};

/*==================================================================================================
  FUNCTION: find_c2c3

  Evaluate the universal functions C2 and C3.
*/
function find_c2c3( psi, tol ) {
  var c2, c3;
  if ( psi > tol ) {
    c2 = ( 1 - Math.cos( Math.sqrt( psi ) ) ) / psi;
    c3 = ( Math.sqrt( psi ) - Math.sin( Math.sqrt( psi ) ) ) / Math.sqrt( psi * psi * psi );
  } else {
    if ( psi < -tol ) {
      c2 = ( 1 - Math.cosh( Math.sqrt( -psi ) ) ) / psi;
      c3 = ( Math.sinh( Math.sqrt( -psi ) ) - Math.sqrt( -psi ) ) / Math.sqrt( -psi * psi * psi );
    } else {
      c2 = .5;
      c3 = 0.1666666666666666667;
    }
  }
  return { c2: c2, c3: c3 };
};  // End of find_c2c3

function linspace( x_start, x_end, x_len ) {
  var dx = ( x_end - x_start ) / ( x_len - 1 );
  var x = [ x_start ];
  for ( var i = 1; i <= x_len; i++ ) {
    x.push( x_start + ( i * dx ) );
  }
  if ( x_len == 1 ) { x = [ x_end ]; }
  return x;
};  // End of linspace


function norm( vec ) {
  // Return norm
  return Math.sqrt( vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2] );
};  // End of norm

function cross( u, v ) {
  // Return vector
  return [ u[1] * v[2] - u[2] * v[1], -u[0] * v[2] + u[2] * v[0], u[0] * v[1] - u[1] * v[0] ];
};  // End of cross

function dot( u, v ) {
  // Return dot product
  return ( u[0] * v[0] + u[1] * v[1] + u[2] * v[2] );
};  // End of dot

function ea2ta( ecc, ea ) {
  // Return true anomaly
  return Math.atan2( Math.sqrt( 1 - ecc * ecc ) * Math.sin( ea ), Math.cos( ea ) - ecc );
};  // End of ea2ta

function ta2ea( ecc, ta ) {
  // Return eccentric anomaly
  return Math.atan2( Math.sqrt( 1 - ecc * ecc ) * Math.sin( ta ), Math.cos( ta ) + ecc );
};  // End of ta2ea

function fkepler( ecc, ea, dm ) {
  // Kepler's equation
  return ( ea - ecc * Math.sin( ea ) - dm )
};  // End of fkepler

function dfkepler( ecc, ea ) {
  // Return derivative of Kepler's equation
  return ( 1 - ecc * Math.cos( ea ) )
};  // End of dfkepler

function ma2ta( ecc, ma ) {
  var ea = ma;
  for ( var iter = 0; iter < 50; iter++ ) {
    var de = -fkepler( ecc, ea, ma ) / dfkepler( ecc, ea );
    ea = ea + de;
    var errorf = Math.abs( fkepler( ecc, ea, ma ) );
    var errorx = Math.abs( de ) / Math.abs( ea );
    if ( ( errorf < 1e-10 ) && ( errorx < 1e-10 ) ) { break; }
  }
  return ea2ta( ecc, ea );
};  // End of ma2ta

function includes(a, b) {
  for ( var i = 0; i < a.length; i++ ) {
    if ( a[i] == b ) {
      return true;
    }
  }
  return false;
}
