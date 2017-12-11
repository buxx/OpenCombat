<?xml version="1.0" encoding="UTF-8"?>
<tileset name="terrain" tilewidth="8" tileheight="8" spacing="1" tilecount="49" columns="7">
 <image source="terrain.png" width="64" height="64"/>
 <tile id="0">
  <properties>
   <property name="name" type="str" value="Grass"/>
   <property name="traversable_by_man" type="bool" value="true"/>
   <property name="traversable_by_vehicle" type="bool" value="true"/>
   <property name="opacity" type="float" value="0.0"/>
   <property name="height" type="float" value="0.0"/>
  </properties>
 </tile>
 <tile id="1">
  <properties>
   <property name="name" type="str" value="Wood wall"/>
   <property name="traversable_by_man" type="bool" value="false"/>
   <property name="traversable_by_vehicle" type="bool" value="false"/>
   <property name="opacity" type="float" value="100.0"/>
   <property name="height" type="float" value="2.0"/>
  </properties>
 </tile>
 <tile id="2">
  <properties>
   <property name="name" type="str" value="Bitume"/>
   <property name="traversable_by_man" type="bool" value="true"/>
   <property name="traversable_by_vehicle" type="bool" value="true"/>
   <property name="opacity" type="float" value="0.0"/>
   <property name="height" type="float" value="0.0"/>
  </properties>
 </tile>
</tileset>
