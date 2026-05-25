// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'srp_proposal.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SrpProposal _$SrpProposalFromJson(Map<String, dynamic> json) => SrpProposal(
  slug: json['slug'] as String,
  name: json['name'] as String,
  description: json['description'] as String,
  schemaJson: json['schema_json'] as Map<String, dynamic>?,
  howItWorks: json['how_it_works'] as String,
  compiledCode: json['compiled_code'] as String,
  status: json['status'] as String,
  intents: (json['intents'] as List<dynamic>).map((e) => e as String).toList(),
  errorLogs: json['error_logs'] as String?,
);

Map<String, dynamic> _$SrpProposalToJson(SrpProposal instance) =>
    <String, dynamic>{
      'slug': instance.slug,
      'name': instance.name,
      'description': instance.description,
      'schema_json': instance.schemaJson,
      'how_it_works': instance.howItWorks,
      'compiled_code': instance.compiledCode,
      'status': instance.status,
      'intents': instance.intents,
      'error_logs': instance.errorLogs,
    };
