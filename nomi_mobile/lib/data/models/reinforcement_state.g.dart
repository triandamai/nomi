// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'reinforcement_state.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ReinforcementState _$ReinforcementStateFromJson(Map<String, dynamic> json) =>
    ReinforcementState(
      slug: json['slug'] as String,
      enrichedDescription: json['enriched_description'] as String,
      additionalRules:
          (json['additional_rules'] as List<dynamic>)
              .map((e) => e as String)
              .toList(),
      learnedPhrases:
          (json['learned_phrases'] as List<dynamic>)
              .map((e) => e as String)
              .toList(),
    );

Map<String, dynamic> _$ReinforcementStateToJson(ReinforcementState instance) =>
    <String, dynamic>{
      'slug': instance.slug,
      'enriched_description': instance.enrichedDescription,
      'additional_rules': instance.additionalRules,
      'learned_phrases': instance.learnedPhrases,
    };
